import { useState, useCallback } from 'react'
import { useNotify } from '../lib/notification'

interface UseApiResult<T> {
  data: T | null
  loading: boolean
  error: Error | null
  execute: (...args: unknown[]) => Promise<T | null>
}

export function useApi<T>(
  apiFn: (...args: unknown[]) => Promise<T>
): UseApiResult<T> {
  const [data, setData] = useState<T | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  const { notifyError } = useNotify()

  const execute = useCallback(
    async (...args: unknown[]) => {
      setLoading(true)
      setError(null)
      try {
        const result = await apiFn(...args)
        setData(result)
        return result
      } catch (err) {
        const e = err instanceof Error ? err : new Error(String(err))
        setError(e)
        notifyError(e.message)
        return null
      } finally {
        setLoading(false)
      }
    },
    [apiFn, notifyError]
  )

  return { data, loading, error, execute }
}
