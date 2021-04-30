//! Defines types used to interact between invoker and valuer
use bitflags::bitflags;
use pom::TestId;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(
    Clone,
    Copy,
    Debug,
    Display,
    EnumString,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Hash,
)]
pub enum StatusKind {
    Queue,
    /// WA, TLE, rejected by teacher, etc
    Rejected,
    /// e.g. Coding Style Violation
    CompilationError,
    // Evaluated,
    Accepted,
    NotSet,
    InternalError,
    Skipped,
}

impl StatusKind {
    pub fn is_success(self) -> bool {
        matches!(self, Self::Accepted)
    }
}

pub mod status_codes {
    macro_rules! declare_code {
        ($code: ident) => {
            pub const $code: &str = stringify!($code);
        };

        ($code: ident, $($codes: ident),+) => {
             declare_code!($code);
             declare_code!($($codes),+);
        };
    }

    // build-related status codes
    declare_code!(
        TOOLCHAIN_SEARCH_ERROR,
        BUILT,
        COMPILATION_TIMED_OUT,
        COMPILER_FAILED
    );

    // per-test status codes
    declare_code!(
        TIME_LIMIT_EXCEEDED,
        RUNTIME_ERROR,
        TEST_PASSED,
        JUDGE_FAULT,
        WRONG_ANSWER,
        PRESENTATION_ERROR,
        LAUNCH_ERROR
    );

    // aggregated status codes
    declare_code!(ACCEPTED, PARTIAL_SOLUTION, BUILD_ERROR);
}

#[derive(Clone, Debug, Serialize, Deserialize, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct Status {
    pub kind: StatusKind,
    pub code: String,
}

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct TestVisibleComponents: u32 {
        /// Test input data
        const TEST_DATA = 1;
        /// Solution stdout & stderr
        const OUTPUT = 2;
        /// Test answer
        const ANSWER = 4;
        /// Test status
        const STATUS = 8;
        /// Resource usage
        const RESOURCE_USAGE = 16;
    }
}

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct SubtaskVisibleComponents: u32 {
        /// Score gained for this subtask
        const SCORE = 1;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct JudgeLogTestRow {
    pub test_id: pom::TestId,
    pub status: Status,
    pub components: TestVisibleComponents,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
pub struct SubtaskId(pub std::num::NonZeroU32);

impl SubtaskId {
    pub fn make(n: u32) -> SubtaskId {
        SubtaskId(std::num::NonZeroU32::new(n).expect("SubtaskId cannot be made from 0"))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct JudgeLogSubtaskRow {
    pub subtask_id: SubtaskId,
    pub score: u32,
    pub components: SubtaskVisibleComponents,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
pub enum JudgeLogKind {
    /// Contains all tests.
    /// Test can be omitted, if staring it was speculation.
    Full,
    /// Contains judge log for contestant
    /// Valuer should respect various restrictions specified in config.
    Contestant,
}

impl JudgeLogKind {
    pub fn as_str(self) -> &'static str {
        match self {
            JudgeLogKind::Full => "full",
            JudgeLogKind::Contestant => "contestant",
        }
    }

    pub fn list() -> impl Iterator<Item = JudgeLogKind> {
        const ALL_KINDS: [JudgeLogKind; 2] = [JudgeLogKind::Contestant, JudgeLogKind::Full];
        ALL_KINDS.iter().copied()
    }
}

/// Judge log from valuer POV
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct JudgeLog {
    pub kind: JudgeLogKind,
    pub tests: Vec<JudgeLogTestRow>,
    pub subtasks: Vec<JudgeLogSubtaskRow>,
    pub score: u32,
    pub is_full: bool,
}

impl Default for JudgeLog {
    fn default() -> JudgeLog {
        JudgeLog {
            kind: JudgeLogKind::Contestant,
            tests: Vec::new(),
            subtasks: Vec::new(),
            score: 0,
            is_full: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemInfo {
    pub tests: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct TestDoneNotification {
    pub test_id: TestId,
    pub test_status: Status,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub enum ValuerResponse {
    Test {
        test_id: TestId,
        live: bool,
    },
    /// Sent when judge log ready
    /// Judge log of each kind must be sent at most once
    JudgeLog(JudgeLog),
    Finish,
    LiveScore {
        score: u32,
    },
}
