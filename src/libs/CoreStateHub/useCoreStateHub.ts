import { useEffect, useState } from 'react'
import { coreStateHub } from '.'
import type { CoreStateStore, CoreStateStoreKey } from './types'

/**
 * Hook to initialize the CoreStateHub once the Core is ready.
 */
export function useCoreStateHub<K extends CoreStateStoreKey>(key: K): CoreStateStore[K]['state'] {
  const [state, setState] = useState<CoreStateStore[K]['state']>(coreStateHub.store[key].state)

  useEffect(() => {
    setState(coreStateHub.store[key].state)
    coreStateHub.addListener<K>(key, setState)

    return () => {
      coreStateHub.removeListener<K>(key, setState)
    }
  }, [key])

  return state
}
