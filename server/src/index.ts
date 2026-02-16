import { Hono } from 'hono'
import { cors } from 'hono/cors'
import { config } from './lib/config'
import { health } from './routes/health'
import { ai } from './routes/ai'

const app = new Hono()

// CORS for Tauri webview origins
app.use(
  '*',
  cors({
    origin: [
      'tauri://localhost',
      'https://tauri.localhost',
      'http://localhost:1420',
    ],
    allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
    allowHeaders: ['Content-Type', 'Authorization'],
  })
)

// Mount routes
app.route('/health', health)
app.route('/ai', ai)

// Root health check
app.get('/', c => c.json({ status: 'ok', server: 'cortex-hono' }))

// Start server
const port = config.honoPort || 0

const server = Bun.serve({
  fetch: app.fetch,
  hostname: '127.0.0.1',
  port,
})

console.log(`Cortex Hono server listening on http://127.0.0.1:${server.port}`)

// Write port to stdout for the Rust sidecar launcher to read
// The format "HONO_PORT:<port>" is parsed by the sidecar manager
console.log(`HONO_PORT:${server.port}`)
