import type { Core } from '../types'

export namespace DaemonClient {
  export interface State {
    is_ready: boolean
    logs: string[]
    status: Core.DashboardStatus
  }
}
