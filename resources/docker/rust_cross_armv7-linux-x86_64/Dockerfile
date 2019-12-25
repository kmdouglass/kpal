FROM rust:1.39-buster as base

# Install the cross-compiler
RUN apt-get update \
 && apt-get install -y \
    gcc-arm-linux-gnueabihf \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists*

FROM base as openssl_builder

ARG OPEN_SSL_BUILD_DIR=/tmp/openssl
ARG OPEN_SSL_INSTALL_DIR=/opt/openssl
ARG OPEN_SSL_REPO=https://github.com/openssl/openssl.git
ARG OPEN_SSL_TAG=OpenSSL_1_1_1d

# Clone the OpenSSL repository
RUN mkdir -p ${OPEN_SSL_BUILD_DIR} ${OPEN_SSL_INSTALL_DIR} \
 && git clone --branch ${OPEN_SSL_TAG} ${OPEN_SSL_REPO} ${OPEN_SSL_BUILD_DIR}

WORKDIR ${OPEN_SSL_BUILD_DIR}

# Compile and install OpenSSL for ARMv7
RUN ./Configure linux-generic32 \
                shared \
                --prefix=${OPEN_SSL_INSTALL_DIR} \
                --openssldir=${OPEN_SSL_INSTALL_DIR}/openssl \
                --cross-compile-prefix=/usr/bin/arm-linux-gnueabihf- \
 && make depend \
 && make \
 && make install

FROM base as kpal_builder

ENV OPEN_SSL_INSTALL_DIR=/opt/openssl
ENV ARMV7_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_LIB_DIR=${OPEN_SSL_INSTALL_DIR}/lib
ENV ARMV7_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_INCLUDE_DIR=${OPEN_SSL_INSTALL_DIR}/include

COPY --from=openssl_builder ${OPEN_SSL_INSTALL_DIR} ${OPEN_SSL_INSTALL_DIR}

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    docker.io \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists*

# Setup Rust for cross-compilation for ARMv7
RUN rustup component add rustfmt \
 && rustup component add clippy \
 && rustup target add armv7-unknown-linux-gnueabihf \
 && mkdir ${HOME}/.cargo \
 && echo '[target.armv7-unknown-linux-gnueabihf]\n\
linker = "arm-linux-gnueabihf-gcc"\n' >> ${HOME}/.cargo/config
