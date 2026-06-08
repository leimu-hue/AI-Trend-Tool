import { contextBridge } from 'electron'

contextBridge.exposeInMainWorld('electronAPI', {
  platform: process.platform,
  version: {
    node: process.versions.node,
    electron: process.versions.electron
  }
})
