FROM alpine AS staging

WORKDIR /staging
ADD mkvdump-x86_64 .
ADD mkvdump-aarch64 .
RUN cp mkvdump-$(uname -m) /usr/local/bin/mkvdump
RUN chmod +x /usr/local/bin/mkvdump

FROM alpine

COPY --from=staging /usr/local/bin/mkvdump /usr/local/bin/

ENTRYPOINT ["mkvdump"]
