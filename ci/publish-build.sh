set -euxo pipefail
export DOCKER_BUILDKIT=1
docker build -f build-env/Dockerfile -t pps-cli --build-arg "BUILD_DATE=$(date)" --build-arg "GIT_HASH=$(git rev-parse HEAD)" .