docker run --rm \
    -v ${PWD}:/code \
    -v ${HOME}/.cargo/registry:/root/.cargo/registry \
    -v ${HOME}/.cargo/git:/root/.cargo/git \
    pathlit/lambda-rust

cp target/lambda/release/bios-checker.zip lambda.zip
