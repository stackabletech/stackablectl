# Using inlined version of ubi8-rust-builder, ends at ## END OF ubi8-rust-builder
# Each modification (apart from removing ONBUILD clauses) should be preceded by a ## PATCH comment explaining the change
#FROM oci.stackable.tech/sdp/ubi8-rust-builder AS builder
FROM registry.access.redhat.com/ubi8/ubi-minimal@sha256:987ae81ce046652ee4a2c3df54dad5e82faa1b078dab5d09f7cfaae11784ed30 AS builder
LABEL maintainer="Stackable GmbH"

# https://github.com/hadolint/hadolint/wiki/DL4006
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# Update image and install everything needed for Rustup & Rust
# hadolint ignore=DL3041
RUN microdnf update --disablerepo=* --enablerepo=ubi-8-appstream-rpms --enablerepo=ubi-8-baseos-rpms -y \
  && rm -rf /var/cache/yum \
  && microdnf install --disablerepo=* --enablerepo=ubi-8-appstream-rpms --enablerepo=ubi-8-baseos-rpms curl findutils gcc gcc-c++ make cmake openssl-devel pkg-config systemd-devel unzip tar xz clang krb5-libs libkadm5 -y \
  && rm -rf /var/cache/yum

WORKDIR /opt/protoc
RUN PROTOC_VERSION=21.5 \
  ARCH=$(arch | sed 's/^aarch64$/aarch_64/') \
  && curl --location --output protoc.zip "https://repo.stackable.tech/repository/packages/protoc/protoc-${PROTOC_VERSION}-linux-${ARCH}.zip" \
  && unzip protoc.zip \
  && rm protoc.zip
ENV PROTOC=/opt/protoc/bin/protoc
WORKDIR /

##PATCH: Install go (for Helm wrapper)
WORKDIR /opt/go
RUN GO_VERSION=1.26.0 \
    ARCH=$(arch | sed 's/^x86_64$/amd64/' | sed 's/^aarch64$/arm64/') \
    && curl --location "https://repo.stackable.tech/repository/packages/go/go${GO_VERSION}.linux-${ARCH}.tar.gz" | tar xvz --strip-components=1 \
    && ln -s /opt/go/bin/go /usr/local/bin
WORKDIR /

# IMPORTANT
# If you change the toolchain version here, make sure to also change the "rust_version"
# property in operator-templating/repositories.yaml
# hadolint ignore=SC1091
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.68.2
## PATCH: cargo-cyclonedx@0.3.7 uses libcargo 0.64 (Cargo 1.63), while our build uses features from Cargo 1.64
## See https://github.com/CycloneDX/cyclonedx-rust-cargo/discussions/416
#  && . "$HOME/.cargo/env" && cargo install --locked cargo-cyclonedx@0.3.7 cargo-auditable@0.6.1

# Build artifacts will be available in /app.
RUN mkdir /app

## PATCH: docker files are in docker/
COPY docker/ubi8-rust-builder/copy_artifacts.sh /

WORKDIR /src
COPY . /src

## PATCH: disabled cyclonedx
# hadolint ignore=SC1091
RUN . "$HOME/.cargo/env" && cargo build --release --workspace
# ONBUILD RUN . "$HOME/.cargo/env" && cargo auditable build --release --workspace && cargo cyclonedx --output-pattern package --all --output-cdx

# Copy the "interesting" files into /app.
RUN find /src/target/release \
                -regextype egrep \
                # The interesting binaries are all directly in ${BUILD_DIR}.
                -maxdepth 1 \
                # Well, binaries are executable.
                -executable \
                # Well, binaries are files.
                -type f \
                # Filter out tests.
                ! -regex ".*\-[a-fA-F0-9]{16,16}$" \
                # Copy the matching files into /app.
                -exec /copy_artifacts.sh {} \;

RUN echo "The following files will be copied to the runtime image: $(ls /app)"
## END OF ubi8-rust-builder

# hadolint ignore=DL3006
FROM registry.access.redhat.com/ubi8/ubi-minimal AS operator

ARG VERSION
ARG RELEASE="1"

LABEL name="Stackable Cockpit" \
  maintainer="info@stackable.de" \
  vendor="Stackable GmbH" \
  version="${VERSION}" \
  release="${RELEASE}" \
  summary="Deploy and manage Stackable clusters." \
  description="Deploy and manage Stackable clusters."

# Update image
# hadolint ignore=DL3041
RUN microdnf install -y yum \
  && yum -y update-minimal --security --sec-severity=Important --sec-severity=Critical \
  && yum clean all \
  && microdnf clean all

# Install kerberos client libraries
# hadolint ignore=DL3041
RUN microdnf install -y krb5-libs libkadm5 && microdnf clean all

#COPY LICENSE /licenses/LICENSE

COPY --from=builder /app/* /usr/local/bin/

RUN groupadd -g 1000 stackable && adduser -u 1000 -g stackable -c 'Stackable Operator' stackable

USER stackable:stackable

ENTRYPOINT ["stackable-cockpitd"]
CMD []
