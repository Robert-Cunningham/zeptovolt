import { Html, Head, Main, NextScript } from "next/document"
import LogRocket from "logrocket"
import { useEffect } from "react"

export default function Document() {
  return (
    <Html lang="en">
      <Head />
      <body>
        <Main />
        <NextScript />
        <Analytics />
      </body>
    </Html>
  )
}

const Analytics = () => {
  useEffect(() => {
    if (process.env.NODE_ENV === "production") {
      LogRocket.init("phqm4b/zeptovolt")
    }
  }, [])

  return <></>
}
