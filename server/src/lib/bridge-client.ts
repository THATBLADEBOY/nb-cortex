/// HTTP client for the Rust axum bridge.
/// Uses the per-session bearer token for authentication.

import { config } from './config'

interface ApiKeyResponse {
  key: string
}

/**
 * Fetch an API key from the Rust bridge (reads from OS keychain).
 * Returns null if the key is not found.
 */
export async function getApiKey(service: string): Promise<string | null> {
  const url = `${config.bridgeUrl}/api-key/${encodeURIComponent(service)}`

  const response = await fetch(url, {
    headers: {
      Authorization: `Bearer ${config.bridgeToken}`,
    },
  })

  if (response.status === 404) {
    return null
  }

  if (!response.ok) {
    throw new Error(
      `Bridge returned ${response.status} for service "${service}"`
    )
  }

  const data = (await response.json()) as ApiKeyResponse
  return data.key
}
