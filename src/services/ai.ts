import { useQuery } from '@tanstack/react-query'
import { logger } from '@/lib/logger'
import { honoFetch } from '@/lib/hono-client'

export const aiQueryKeys = {
  all: ['ai'] as const,
  status: () => [...aiQueryKeys.all, 'status'] as const,
}

interface AiStatus {
  available: boolean
  message: string
}

/** Check whether the AI service is available. */
export function useAiStatus() {
  return useQuery({
    queryKey: aiQueryKeys.status(),
    queryFn: async (): Promise<AiStatus> => {
      logger.debug('Checking AI status')
      return honoFetch<AiStatus>('/ai/status')
    },
    staleTime: 1000 * 60,
    retry: false,
  })
}
