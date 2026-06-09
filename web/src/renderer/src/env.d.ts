/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_BASE_URL: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

interface ElectronClipboard {
  writeText: (text: string) => Promise<void>
  readText: () => Promise<string>
}

interface ElectronAPI {
  platform: string
  version: {
    node: string
    electron: string
  }
  clipboard: ElectronClipboard
}

interface Window {
  electronAPI: ElectronAPI
}
