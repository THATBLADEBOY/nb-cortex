/// Configuration read from environment variables set by the Rust sidecar launcher.

export const config = {
  /** URL of the Rust axum bridge (e.g., "http://127.0.0.1:12345") */
  bridgeUrl: process.env.CORTEX_BRIDGE_URL ?? 'http://127.0.0.1:9999',
  /** Bearer token for authenticating with the bridge */
  bridgeToken: process.env.CORTEX_BRIDGE_TOKEN ?? '',
  /** Port for the Hono server to listen on */
  honoPort: parseInt(process.env.CORTEX_HONO_PORT ?? '0', 10),
} as const
