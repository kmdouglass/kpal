FROM multiarch/debian-debootstrap:armhf-buster-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends libssl1.1 \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists* \
 && mkdir -p /opt

COPY entrypoint.sh /opt/run_tests.sh

ENTRYPOINT [ "bash", "/opt/run_tests.sh" ]
