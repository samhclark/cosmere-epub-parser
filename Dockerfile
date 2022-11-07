FROM docker.io/library/rust:1.65-slim-buster as builder 
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .

FROM docker.io/library/debian:buster-slim
RUN apt-get update --quiet --assume-yes \
    && apt-get upgrade --quiet --assume-yes \
    && apt-get install dumb-init --quiet --assume-yes

COPY --from=builder /usr/local/cargo/bin/cosmere-epub-parser /usr/local/bin/cosmere-epub-parser
COPY ./the-bands-of-mourning.epub /the-bands-of-mourning.epub
EXPOSE 8080
ENTRYPOINT ["/usr/bin/dumb-init", "--", "/usr/local/bin/cosmere-epub-parser"]