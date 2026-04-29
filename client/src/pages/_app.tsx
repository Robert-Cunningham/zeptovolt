import '@/styles/globals.css'
import 'react-tooltip/dist/react-tooltip.css'
import LogRocket from 'logrocket'
import type { AppProps } from 'next/app'
import { useEffect } from 'react'

export default function App({ Component, pageProps }: AppProps) {
  useEffect(() => {
    if (process.env.NODE_ENV === 'production') {
      LogRocket.init('phqm4b/zeptovolt')
    }
  }, [])

  return <Component {...pageProps} />
}
