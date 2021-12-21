FROM ubuntu:22.04

LABEL authors="red.avtovo@gmail.com"

RUN apt update &&\
    apt install libpq-dev ca-certificates -y &&\
    update-ca-certificates &&\
    apt clean

COPY ./remote-transmission-bot /opt/

ENV RUST_LOG="info,transmission_rpc=warn"

WORKDIR /opt
CMD ["./remote-transmission-bot"]