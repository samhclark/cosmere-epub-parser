FROM docker.io/library/rust:1.65-slim-buster as builder 
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .

FROM docker.io/library/debian:buster-slim
RUN apt-get update --quiet --assume-yes \
    && apt-get upgrade --quiet --assume-yes \
    && apt-get install dumb-init --quiet --assume-yes \
    && mkdir /assets

COPY --from=builder /usr/local/cargo/bin/cosmere_search_web_server /usr/local/bin/cosmere_search_web_server
COPY ./assets/* /assets/
COPY ./*.epub /
EXPOSE 8080
ENTRYPOINT ["/usr/bin/dumb-init", "--", "/usr/local/bin/cosmere_search_web_server"]