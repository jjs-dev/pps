set -euxo pipefail

# GENERATED FILE DO NOT EDIT
if [ "$GITHUB_REF" = "refs/heads/master" ]
then
  TAG="latest"
elif [ "$GITHUB_REF" = "refs/heads/trying" ]
then
  TAG="dev"
else
  echo "unknown GITHUB_REF: $GITHUB_REF"
  exit 1
fi
echo $GITHUB_TOKEN | docker login ghcr.io -u $GITHUB_ACTOR --password-stdin
docker tag pps ghcr.io/jjs-dev/pps:$TAG
docker push ghcr.io/jjs-dev/pps:$TAG
docker tag pps-server ghcr.io/jjs-dev/pps-server:$TAG
docker push ghcr.io/jjs-dev/pps-server:$TAG