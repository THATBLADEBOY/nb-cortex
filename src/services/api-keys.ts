import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { logger } from '@/lib/logger'
import { commands, type ApiKeyEntry } from '@/lib/tauri-bindings'

export const apiKeysQueryKeys = {
  all: ['api-keys'] as const,
  services: () => [...apiKeysQueryKeys.all, 'services'] as const,
}

/** List all known API key services with their storage status. */
export function useApiKeyServices() {
  return useQuery({
    queryKey: apiKeysQueryKeys.services(),
    queryFn: async (): Promise<ApiKeyEntry[]> => {
      logger.debug('Loading API key services')
      const result = await commands.listApiKeyServices()

      if (result.status === 'error') {
        logger.error('Failed to load API key services', { error: result.error })
        throw new Error(result.error)
      }

      return result.data
    },
    staleTime: 1000 * 60 * 5,
  })
}

/** Store an API key in the OS keychain. */
export function useSetApiKey() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ service, key }: { service: string; key: string }) => {
      logger.debug('Storing API key', { service })
      const result = await commands.setApiKey(service, key)

      if (result.status === 'error') {
        logger.error('Failed to store API key', {
          error: result.error,
          service,
        })
        toast.error('Failed to store API key', { description: result.error })
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: apiKeysQueryKeys.services() })
      toast.success('API key saved')
    },
  })
}

/** Remove an API key from the OS keychain. */
export function useDeleteApiKey() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (service: string) => {
      logger.debug('Deleting API key', { service })
      const result = await commands.deleteApiKey(service)

      if (result.status === 'error') {
        logger.error('Failed to delete API key', {
          error: result.error,
          service,
        })
        toast.error('Failed to delete API key', { description: result.error })
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: apiKeysQueryKeys.services() })
      toast.success('API key removed')
    },
  })
}
