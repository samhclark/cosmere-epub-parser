ARG rust_version
ARG debian_version=bookworm
FROM docker.io/library/rust:${rust_version}-slim-${debian_version} as builder 
WORKDIR /usr/src/myapp
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
COPY . .
RUN apt-get update --quiet --assume-yes \
    && apt-get install --quiet --assume-yes mold \
    && cargo install --locked --path .

ARG debian_version
FROM docker.io/library/debian:${debian_version}-slim
RUN apt-get update --quiet --assume-yes \
    && apt-get install dumb-init --quiet --assume-yes \
    && mkdir /assets

COPY --from=builder /usr/local/cargo/bin/cosmere_search_web_server /usr/local/bin/cosmere_search_web_server
COPY ./assets/* /assets/
COPY ./input.json /input.json
EXPOSE 8080
EXPOSE 9091
ENTRYPOINT ["/usr/bin/dumb-init", "--", "/usr/local/bin/cosmere_search_web_server"]
