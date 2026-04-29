import Head from "next/head"
import { useContext, useEffect, useState } from "react"
import useSWR from "swr"
import React from "react"
import Highlighter from "react-highlight-words"
import { Tooltip } from "react-tooltip"
import { ErrorBoundary } from "react-error-boundary"

const SearchContext = React.createContext("")

function escapeRegexLiteral(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")
}

function safeHighlightSearchWords(searchText: string): string[] {
  return searchText
    .split(/\s+/)
    .filter(Boolean)
    .map((word) => {
      try {
        new RegExp(word)
        return word
      } catch {
        return escapeRegexLiteral(word)
      }
    })
}

const Home = () => {
  return (
    <>
      <Head>
        <title>Zeptovolt</title>
        <link rel="preconnect" href={API_ENDPOINT} crossOrigin="anonymous" />
      </Head>
      <main>
        <div className="py-8 bg-slate-100 min-h-screen w-screen">
          <CentralColumn></CentralColumn>
        </div>
      </main>
    </>
  )
}

const ErrorFallback = ({ error }: { error: Error }) => {
  if (error.message.includes("Invalid regular expression")) {
    return (
      <div className="mx-auto">
        Looks like you have an invalid regular expression in your search query.
        <pre>{error.message}</pre>
      </div>
    )
  } else {
    return (
      <div className="mx-auto">
        <p>Something went wrong:</p>
        <pre>{error.message}</pre>
      </div>
    )
  }
}

function formatUSDPrice(value: number): string {
  const options: Intl.NumberFormatOptions = {
    style: "currency",
    currency: "USD",
  }

  // Check if the value is less than a cent
  if (value < 0.01) {
    // Calculate the number of significant digits after the decimal point
    const significantDigits = Math.ceil(-Math.log10(value))

    // Set the minimum and maximum fraction digits to the significant digits
    options.minimumFractionDigits = significantDigits
    options.maximumFractionDigits = significantDigits
  } else {
    // Set the minimum and maximum fraction digits to 2 (for cents)
    options.minimumFractionDigits = 2
    options.maximumFractionDigits = 2
  }

  // Format the value using Intl.NumberFormat
  const formatter = new Intl.NumberFormat("en-US", options)
  return formatter.format(value)
}

function shortenNumber(num: number): string | number {
  if (typeof num !== "number" || isNaN(num)) {
    return num
  }

  let suffix = ""
  let divisor = 1

  if (num >= 1_000_000) {
    suffix = "M"
    divisor = 1_000_000
  } else if (num >= 1_000) {
    suffix = "k"
    divisor = 1_000
  }

  const shortNum = Math.round(num / divisor)
  return shortNum.toString() + suffix
}

function ResistorRow({
  image_url,
  description,
  manufacturer_id,
  price,
  stock,
  basic_or_extended,
  datasheet_url,
  lcsc_id,
  first,
  last,
}: Part & { first: boolean; last: boolean }) {
  const search_strings = safeHighlightSearchWords(useContext(SearchContext))

  return (
    <tr
      className={
        "bg-white border py-2 " +
        (first ? " rounded-t-md " : "") +
        (last ? " rounded-b-md " : "")
      }
    >
      <td className="px-2 mr-0">
        <a href={datasheet_url}>
          <p className="text-gray-600">
            <Highlighter
              searchWords={search_strings}
              textToHighlight={manufacturer_id}
            ></Highlighter>
          </p>
        </a>
      </td>
      <td className="px-2">
        <button
          type="button"
          {...ttProps}
          data-tooltip-id="lcsc"
          data-tooltip-content="Click to copy"
          className="text-gray-600 hover:bg-yellow-100 cursor-pointer inline-flex active:bg-green-100 transition-colors transition-none duration-100 bg-transparent border-0 p-0"
          onClick={() => {
            navigator.clipboard.writeText(lcsc_id)
          }}
        >
          <Highlighter searchWords={search_strings} textToHighlight={lcsc_id}>
            {" "}
          </Highlighter>
        </button>
      </td>
      <td className="px-2 w-16 h-16">
        {image_url && (
          <a
            {...ttProps}
            data-tooltip-id="image"
            data-image-url={`https://assets.lcsc.com/images/lcsc/224x224/${image_url}`}
          >
            <img
              alt={description}
              className="rounded-sm"
              src={`https://assets.lcsc.com/images/lcsc/96x96/${image_url}`}
            ></img>
          </a>
        )}
      </td>
      <td className="px-2">
        <p className="text-gray-600">
          <Highlighter
            searchWords={search_strings}
            textToHighlight={description}
          >
            {" "}
          </Highlighter>
        </p>
      </td>
      <td className="px-2">
        <p className="text-gray-600">
          <a
            {...ttProps}
            data-tooltip-id="bore"
            data-tooltip-content={
              basic_or_extended.toLowerCase() + " part type"
            }
          >
            {basic_or_extended.at(0)?.toUpperCase()}
          </a>
        </p>
      </td>
      <td className="px-2">
        <p className="text-gray-600">
          <a
            {...ttProps}
            data-tooltip-id="price"
            data-tooltip-content={"$" + price}
          >
            {formatUSDPrice(price)}
          </a>
        </p>
      </td>
      <td className="px-2">
        <p className="text-gray-600">
          <a
            {...ttProps}
            data-tooltip-id="stock"
            data-tooltip-content={stock + " in stock"}
          >
            {shortenNumber(stock)}
          </a>
        </p>
      </td>
    </tr>
  )
}

const ttProps = {
  "data-tooltip-delay-show": 100,
  className: "cursor-pointer",
}

const ttElProps = {
  style: { backgroundColor: "rgb(0, 0, 0)", opacity: 1 },
}

const API_ENDPOINT =
  process.env.NEXT_PUBLIC_API_ENDPOINT ??
  (process.env.NODE_ENV === "development"
    ? "http://localhost:8090"
    : "https://api.zeptovolt.com")

const CentralColumn = () => {
  const [text, setText] = useState<string>("")
  const [results, setResults] = useState<Part[]>([])
  const [searchMetadata, setSearchMetadata] = useState<
    SearchResponseMetadata | undefined
  >()

  const dbText = useDebounce(text, 100)

  const { response, controller } = useCancelableSWR<SearchResponse>(
    `${API_ENDPOINT}/search?${new URLSearchParams({ q: dbText })}`
  )
  const { data, isLoading, error } = response

  useEffect(() => {
    if (data && !isLoading && !error) {
      const { info, results } = data.body
      setResults(results)
      setSearchMetadata({
        partsSearched: info.parts_searched,
        totalResults: info.total_results,
        serverTimeMs: info.server_time_ms,
        clientTimeMs: data.clientTimeMs,
      })
    }
  }, [data, isLoading, error, setResults])

  const cancelLastAndSetText = (newText: string) => {
    controller.abort()
    setSearchMetadata(undefined)
    setText(newText)
  }

  const clientTime = searchMetadata?.clientTimeMs
  const serverTime = searchMetadata?.serverTimeMs
  const totalResults = searchMetadata?.totalResults ?? results.length
  const networkTime =
    clientTime !== undefined && serverTime !== undefined
      ? Math.max(clientTime - serverTime, 0)
      : undefined
  const totalTime =
    clientTime !== undefined && serverTime !== undefined
      ? Math.max(clientTime, serverTime)
      : clientTime ?? serverTime
  const searchStatus =
    text === dbText && totalTime !== undefined
      ? `${
          searchMetadata?.partsSearched !== undefined
            ? `Searched ${shortenNumber(
                searchMetadata.partsSearched
              )} JLCPCB parts in ${totalTime}ms${
                serverTime !== undefined && networkTime !== undefined
                  ? ` (${serverTime}ms server + ${networkTime}ms network)`
                  : ""
              }. `
            : `Search completed in ${totalTime}ms. `
        }${
          totalResults > results.length
            ? `Showing first ${results.length} of ${shortenNumber(
                totalResults
              )} results.`
            : `${totalResults} results.`
        }`
      : ""

  return (
    <div className="md:max-w-4xl mx-auto flex flex-col gap-4">
      <SearchContext.Provider value={text}>
        <div className="relative flex min-h-6 items-center justify-center px-3">
          <p className="text-gray-800 font-semibold text-md text-center">
            Fast JLCPCB Parts Search
          </p>
          <a
            aria-label="GitHub repository"
            className="absolute right-3 text-gray-500 hover:text-gray-700"
            href="https://github.com/Robert-Cunningham/zeptovolt"
            rel="noreferrer"
            target="_blank"
          >
            <svg
              aria-hidden="true"
              className="h-5 w-5"
              fill="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                clipRule="evenodd"
                d="M12 2C6.477 2 2 6.477 2 12c0 4.418 2.865 8.166 6.839 9.489.5.092.682-.217.682-.482 0-.237-.009-.866-.014-1.7-2.782.604-3.369-1.342-3.369-1.342-.454-1.154-1.11-1.462-1.11-1.462-.908-.621.069-.608.069-.608 1.004.07 1.532 1.031 1.532 1.031.892 1.529 2.341 1.087 2.91.832.091-.647.349-1.087.635-1.337-2.221-.253-4.555-1.111-4.555-4.944 0-1.092.39-1.985 1.03-2.683-.103-.253-.446-1.27.098-2.647 0 0 .84-.269 2.75 1.025A9.564 9.564 0 0 1 12 6.835c.85.004 1.705.115 2.504.337 1.909-1.294 2.747-1.025 2.747-1.025.546 1.377.203 2.394.1 2.647.64.698 1.028 1.591 1.028 2.683 0 3.842-2.337 4.688-4.566 4.936.359.309.678.919.678 1.852 0 1.337-.012 2.416-.012 2.744 0 .267.18.579.688.481C19.138 20.162 22 16.416 22 12c0-5.523-4.477-10-10-10Z"
                fillRule="evenodd"
              />
            </svg>
          </a>
        </div>
        <SearchBox {...{ text, setText: cancelLastAndSetText }}></SearchBox>
        <div
          aria-live="polite"
          className="h-10 overflow-hidden px-3 text-gray-400 text-sm leading-5 sm:h-5"
          title={searchStatus}
        >
          <p className="max-h-10 overflow-hidden sm:truncate">
            {searchStatus}
          </p>
        </div>
        <ErrorBoundary FallbackComponent={ErrorFallback}>
          <Tooltip id="stock" {...ttElProps} />
          <Tooltip id="price" {...ttElProps} />
          <Tooltip id="bore" {...ttElProps} />
          <Tooltip id="lcsc" {...ttElProps} />
          <Tooltip
            id="image"
            {...ttElProps}
            render={({ activeAnchor }) => {
              const url = activeAnchor?.getAttribute("data-image-url")
              return url ? (
                <img alt="" className="rounded-sm" src={url}></img>
              ) : null
            }}
          />
          <table className="table-auto rounded-md">
            <tbody>
              {results.map((part: Part, i: number) => (
                <ResistorRow
                  key={part.lcsc_id}
                  first={i === 0}
                  last={i === results.length - 1}
                  {...part}
                ></ResistorRow>
              ))}
            </tbody>
          </table>
        </ErrorBoundary>
      </SearchContext.Provider>
    </div>
  )
}

const SearchBox = ({
  text,
  setText,
}: {
  text: string
  setText: (a0: string) => void
}) => {
  return (
    <input
      value={text}
      onChange={(e) => setText(e.target.value)}
      className="w-full h-8 rounded-lg text-sm p-5 border"
      placeholder="10k 0603 resistor basic"
    ></input>
  )
}

interface Part {
  description: string
  lcsc_id: string
  manufacturer_id: string
  image_url: string | null
  datasheet_url: string
  basic_or_extended: string
  price: number
  stock: number
}

interface SearchResponse {
  results: Part[]
  info: SearchResponseInfo
}

interface SearchResponseInfo {
  parts_searched: number
  total_results: number
  server_time_ms: number
}

interface SearchResponseMetadata {
  partsSearched: number
  totalResults: number
  serverTimeMs: number
  clientTimeMs: number
}

export default Home

// Hook
// T is a generic type for value parameter, our case this will be string
function useDebounce<T>(value: T, delay: number): T {
  // State and setters for debounced value
  const [debouncedValue, setDebouncedValue] = useState<T>(value)
  useEffect(
    () => {
      // Update debounced value after delay
      const handler = setTimeout(() => {
        setDebouncedValue(value)
      }, delay)
      // Cancel the timeout if value changes (also on delay change or unmount)
      // This is how we prevent debounced value from updating if value is changed ...
      // .. within the delay period. Timeout gets cleared and restarted.
      return () => {
        clearTimeout(handler)
      }
    },
    [value, delay] // Only re-call effect if value or delay changes
  )
  return debouncedValue
}

function useCancelableSWR<T>(key: string) {
  const controller = React.useMemo(() => new AbortController(), [key])

  return {
    response: useSWR<TimedResponse<T>>(key, async (url: string) => {
      const startedAt = performance.now()
      const response = await fetch(url, { signal: controller.signal })
      const body = (await response.json()) as T

      return {
        body,
        clientTimeMs: Math.max(Math.round(performance.now() - startedAt), 0),
      }
    }),
    controller,
  }
}

interface TimedResponse<T> {
  body: T
  clientTimeMs: number
}
