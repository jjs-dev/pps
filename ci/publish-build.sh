set -euxo pipefail
export DOCKER_BUILDKIT=1
docker build -f build-env/Dockerfile -t pps-cli .