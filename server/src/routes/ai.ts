import { Hono } from 'hono'

const ai = new Hono()

// Placeholder: AI/LLM routes will be added here
ai.get('/status', c => {
  return c.json({ available: false, message: 'AI routes not yet implemented' })
})

export { ai }
