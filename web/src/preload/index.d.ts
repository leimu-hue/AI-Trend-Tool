export interface ElectronAPI {
  platform: string
  version: {
    node: string
    electron: string
  }
}

declare global {
  interface Window {
    electronAPI: ElectronAPI
  }
}
