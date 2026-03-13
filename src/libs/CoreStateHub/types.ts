import type { Cloud } from '@core/Cloud/types'
import type { DaemonClient } from '@core/DaemonClient/types'
import type { Scanner } from '@core/Scanner/types'

type CoreState = {
  cloud: Cloud.State
  dashboard: DaemonClient.State
  scanner: Scanner.State
}

export type CoreStateStore = {
  [K in keyof CoreState]: {
    listeners: CoreStateListener<K>[]
    state: CoreState[K] | undefined
  }
}

export type CoreStateStoreKey = keyof CoreStateStore

export type CoreStateListener<K extends CoreStateStoreKey> = (nextState: CoreState[K]) => void
