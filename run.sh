unzip -o \
    lambda.zip \
    -d /tmp/lambda && \
docker run \
  -i \
  -e LATEST_VER=4801 \
  --rm \
  -v /tmp/lambda:/var/task \
  lambci/lambda:provided.al2 \
  hello.handler \
  '{}'
