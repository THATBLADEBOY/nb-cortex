/**
 * HTTP client for the Hono sidecar server.
 *
 * Discovers the Hono port via the `server-ready` Tauri event or the
 * `getServerStatus` command, then provides a typed fetch helper.
 */

import { listen } from '@tauri-apps/api/event'
import { commands } from '@/lib/tauri-bindings'
import { logger } from '@/lib/logger'

let honoPort: number | null = null
let portPromise: Promise<number> | null = null

/**
 * Initialize the Hono client by listening for the server-ready event.
 * Call this once during app startup.
 */
export function initHonoClient(): void {
  portPromise = new Promise<number>(resolve => {
    // First, check if the server is already running
    commands.getServerStatus().then(status => {
      if (status.running && status.port) {
        honoPort = status.port
        logger.info('Hono server already running', { port: honoPort })
        resolve(honoPort)
      }
    })

    // Also listen for the server-ready event (in case it starts after we check)
    listen<{ port: number }>('server-ready', event => {
      honoPort = event.payload.port
      logger.info('Hono server ready', { port: honoPort })
      resolve(honoPort)
    })
  })
}

/**
 * Get the Hono server base URL. Waits for the server to be ready if needed.
 */
async function getBaseUrl(): Promise<string> {
  if (honoPort) {
    return `http://127.0.0.1:${honoPort}`
  }

  if (!portPromise) {
    initHonoClient()
  }

  // portPromise is guaranteed to be set after initHonoClient()
  const port = await (portPromise as Promise<number>)
  return `http://127.0.0.1:${port}`
}

/**
 * Typed fetch helper for the Hono server.
 * Automatically prepends the base URL and parses JSON responses.
 */
export async function honoFetch<T>(
  path: string,
  options?: RequestInit
): Promise<T> {
  const baseUrl = await getBaseUrl()
  const url = `${baseUrl}${path}`

  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  })

  if (!response.ok) {
    throw new Error(
      `Hono request failed: ${response.status} ${response.statusText}`
    )
  }

  return response.json() as Promise<T>
}
