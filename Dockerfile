FROM rust:1.77.2

WORKDIR /usr/src/rain_sg
COPY . .
# ENV TELOXIDE_TOKEN=<TOKEN HERE>

RUN cargo install --path .

CMD ["rain_sg"]