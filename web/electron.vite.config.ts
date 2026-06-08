import { defineConfig, externalizeDepsPlugin } from 'electron-vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { resolve } from 'path'

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin()]
  },
  preload: {
    plugins: [externalizeDepsPlugin()]
  },
  renderer: {
    plugins: [
      react({
        babel: {
          plugins: [['babel-plugin-react-compiler', { target: '19' }]]
        }
      }),
      tailwindcss()
    ],
    resolve: {
      alias: {
        '@': resolve('src/renderer/src')
      }
    }
  }
})
