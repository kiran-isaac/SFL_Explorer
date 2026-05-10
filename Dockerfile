FROM kiranisturt/wasm_pack:1.81 AS wasm-builder

COPY prelude.sfl /build/prelude.sfl
COPY wasm_lib /build/wasm_lib
COPY sfl_frontend_lib /build/sfl_frontend_lib

ENV PATH="/root/.cargo/bin:${PATH}"

# Build the wasm library
WORKDIR /build/wasm_lib
RUN wasm-pack build --target web

# Build the frontend
FROM node:22-alpine AS frontend-builder

COPY --from=wasm-builder /build/wasm_lib/pkg /build/wasm_lib/pkg
COPY definition.md /build/definition.md
COPY frontend /build/frontend
COPY examples /build/examples

WORKDIR /build/frontend
# Remove node modules if they exist
RUN rm -rf node_modules
RUN npm ci
RUN npm run build