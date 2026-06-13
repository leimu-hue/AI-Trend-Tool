export interface ElectronAPI {
  platform: string
  version: {
    node: string
    electron: string
  }
  clipboard: {
    writeText: (text: string) => Promise<void>
  }
}

declare global {
  interface Window {
    electronAPI: ElectronAPI
  }
}
