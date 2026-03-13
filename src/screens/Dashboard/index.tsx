import { CloudModule } from '@core/Cloud'
import { Core } from '@core/types'
import { useInterval } from '@hooks/useInterval'
import { useCoreStateHub } from '@libs/CoreStateHub/useCoreStateHub'
import { invoke } from '@tauri-apps/api/core'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { DashboardScreenComponent } from './Component'

interface History {
  lastCloudUpdate: string | null
  lastFullScan: string | null
  lastPartialScan: string | null
}

export function Dashboard() {
  const cloudState = useCoreStateHub('cloud')
  const dashboardState = useCoreStateHub('dashboard')
  const [history, setHistory] = useState<History | undefined>(undefined)

  const checkCloudUpdate = useCallback(CloudModule.checkCloudUpdate, [])
  const startCloudUpdate = useCallback(CloudModule.startCloudUpdate, [])

  useInterval(checkCloudUpdate, 60_000, cloudState?.status === Core.ModuleStatus.Running)

  useEffect(() => {
    invoke<History>('get_history').then(setHistory)
  }, [])

  const daemonLogs: Core.Log[] | undefined = useMemo(
    () =>
      dashboardState?.logs.map(line => ({
        date: new Date().toISOString(),
        message: line,
        type: 'stdout' as const,
      })),
    [dashboardState?.logs],
  )

  return (
    <DashboardScreenComponent
      cloudState={cloudState}
      daemonClientState={dashboardState}
      daemonLogs={daemonLogs}
      history={history}
      onStartCloudUpdate={startCloudUpdate}
    />
  )
}
