FROM alpine AS staging

WORKDIR /staging
COPY artifacts .
RUN cp mkvdump-$(uname -m)/mkvdump /usr/local/bin/mkvdump
RUN chmod +x /usr/local/bin/mkvdump

FROM alpine

COPY --from=staging /usr/local/bin/mkvdump /usr/local/bin/

ENTRYPOINT ["mkvdump"]
