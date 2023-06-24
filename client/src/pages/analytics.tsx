import LogRocket from "logrocket"
import { useEffect } from "react"
export const Analytics = () => {
  useEffect(() => {
    if (process.env.NODE_ENV === "production") {
      LogRocket.init("phqm4b/zeptovolt")
    }
  }, [])

  return <></>
}
