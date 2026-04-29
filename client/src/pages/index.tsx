import Head from "next/head"
import Image from "next/image"
import { useContext, useEffect, useState } from "react"
import useSWR from "swr"
import React from "react"
import Highlighter from "react-highlight-words"
import { Tooltip } from "react-tooltip"
import { ErrorBoundary } from "react-error-boundary"

const SearchContext = React.createContext("")
const highlightClassName = "rounded-sm bg-amber-50 px-0.5 text-slate-700"

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
      <main className="min-h-screen bg-[#f6f7f9] text-slate-900">
        <div className="min-h-screen w-full px-4 py-6 sm:px-6">
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

function formatDurationUs(durationUs: number): string {
  const roundedUs = Math.max(Math.round(durationUs), 0)

  if (roundedUs < 1000) {
    return `${roundedUs}us`
  }

  return `${Math.round(roundedUs / 1000)}ms`
}

function hideFailedImage(event: React.SyntheticEvent<HTMLImageElement>): void {
  event.currentTarget.style.display = "none"
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
}: Part) {
  const search_strings = safeHighlightSearchWords(useContext(SearchContext))

  return (
    <tr className="bg-white transition-colors hover:bg-slate-50">
      <td className="px-4 py-3 align-middle">
        <a href={datasheet_url}>
          <p className="whitespace-nowrap font-mono text-sm text-slate-700">
            <Highlighter
              highlightClassName={highlightClassName}
              searchWords={search_strings}
              textToHighlight={manufacturer_id}
            ></Highlighter>
          </p>
        </a>
      </td>
      <td className="px-4 py-3 align-middle">
        <button
          type="button"
          {...ttProps}
          data-tooltip-id="lcsc"
          data-tooltip-content="Click to copy"
          className="inline-flex cursor-pointer border-0 bg-transparent p-0 font-mono text-sm text-slate-600 transition-colors duration-100 hover:bg-amber-50 active:bg-green-100"
          onClick={() => {
            navigator.clipboard.writeText(lcsc_id)
          }}
        >
          <Highlighter
            highlightClassName={highlightClassName}
            searchWords={search_strings}
            textToHighlight={lcsc_id}
          >
            {" "}
          </Highlighter>
        </button>
      </td>
      <td className="h-16 w-16 px-4 py-3 align-middle">
        {image_url && (
          <a
            {...ttProps}
            data-tooltip-id="image"
            data-image-url={`https://assets.lcsc.com/images/lcsc/224x224/${image_url}`}
          >
            {/* eslint-disable-next-line @next/next/no-img-element -- LCSC blocks Next's server-side image optimizer fetches. */}
            <img
              alt=""
              className="rounded-sm"
              decoding="async"
              height={48}
              loading="lazy"
              onError={hideFailedImage}
              src={`https://assets.lcsc.com/images/lcsc/96x96/${image_url}`}
              width={48}
            ></img>
          </a>
        )}
      </td>
      <td className="min-w-[22rem] px-4 py-3 align-middle">
        <p className="text-xs leading-5 text-slate-600">
          <Highlighter
            highlightClassName={highlightClassName}
            searchWords={search_strings}
            textToHighlight={description}
          >
            {" "}
          </Highlighter>
        </p>
      </td>
      <td className="px-4 py-3 text-center align-middle">
        <p className="font-mono text-sm text-slate-600">
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
      <td className="px-4 py-3 text-right align-middle">
        <p className="whitespace-nowrap font-mono text-sm tabular-nums text-slate-600">
          <a
            {...ttProps}
            data-tooltip-id="price"
            data-tooltip-content={"$" + price}
          >
            {formatUSDPrice(price)}
          </a>
        </p>
      </td>
      <td className="px-4 py-3 text-right align-middle">
        <p className="whitespace-nowrap font-mono text-sm tabular-nums text-slate-600">
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
  process.env.NODE_ENV === "development"
    ? "http://localhost:8090"
    : process.env.NEXT_PUBLIC_API_ENDPOINT ?? "https://api.zeptovolt.com"

const CentralColumn = () => {
  const [text, setText] = useState<string>("")
  const [results, setResults] = useState<Part[]>([])
  const [searchMetadata, setSearchMetadata] = useState<
    SearchResponseMetadata | undefined
  >()

  const dbText = useDebounce(text, 100)
  const hasQuery = text.trim().length > 0
  const queryReady = hasQuery && text === dbText

  const { response, controller } = useCancelableSWR<SearchResponse>(
    dbText.trim() === ""
      ? null
      : `${API_ENDPOINT}/search?${new URLSearchParams({ q: dbText })}`
  )
  const { data, isLoading, error } = response

  useEffect(() => {
    if (dbText.trim() === "") {
      setResults([])
      setSearchMetadata(undefined)
    }
  }, [dbText, setResults])

  useEffect(() => {
    if (data && !isLoading && !error) {
      const { info, results } = data.body
      setResults(results)
      setSearchMetadata({
        partsSearched: info.parts_searched,
        totalResults: info.total_results,
        serverTimeUs: info.server_time_us,
        clientTimeUs: data.clientTimeUs,
      })
    }
  }, [data, isLoading, error, setResults])

  const cancelLastAndSetText = (newText: string) => {
    controller.abort()
    setText(newText)
  }

  const clientTimeUs = searchMetadata?.clientTimeUs
  const serverTimeUs = searchMetadata?.serverTimeUs
  const totalResults = searchMetadata?.totalResults ?? results.length
  const networkTimeUs =
    clientTimeUs !== undefined && serverTimeUs !== undefined
      ? Math.max(clientTimeUs - serverTimeUs, 0)
      : undefined
  const totalTimeUs =
    clientTimeUs !== undefined && serverTimeUs !== undefined
      ? Math.max(clientTimeUs, serverTimeUs)
      : clientTimeUs ?? serverTimeUs
  const searchStatus =
    hasQuery && (!queryReady || isLoading)
      ? "Searching..."
      : hasQuery && queryReady && totalTimeUs !== undefined
      ? `${
          searchMetadata?.partsSearched !== undefined
            ? `Searched ${shortenNumber(
                searchMetadata.partsSearched
              )} JLCPCB parts in ${formatDurationUs(totalTimeUs)}${
                serverTimeUs !== undefined && networkTimeUs !== undefined
                  ? ` (${formatDurationUs(
                      serverTimeUs
                    )} server + ${formatDurationUs(networkTimeUs)} network)`
                  : ""
              }. `
            : `Search completed in ${formatDurationUs(totalTimeUs)}. `
        }${
          totalResults > results.length
            ? `Showing first ${results.length} of ${shortenNumber(
                totalResults
              )} results.`
            : `${totalResults} results.`
        }`
      : ""
  const showEmptyResults =
    hasQuery && queryReady && !isLoading && !error && results.length === 0

  return (
    <div
      className={
        hasQuery
          ? "mx-auto flex min-h-[calc(100vh-3rem)] w-full max-w-6xl flex-col gap-4"
          : "mx-auto flex min-h-[calc(100vh-3rem)] w-full max-w-3xl flex-col items-center justify-center pb-20"
      }
    >
      <SearchContext.Provider value={text}>
        <div
          className={
            hasQuery
              ? "flex min-h-9 items-center justify-between gap-4"
              : "mb-7 flex flex-col items-center text-center"
          }
        >
          <div
            className={
              hasQuery
                ? "flex items-center gap-2"
                : "flex flex-col items-center gap-3"
            }
          >
            <Image
              alt=""
              className={hasQuery ? "h-7 w-7" : "h-12 w-12"}
              height={48}
              src="/zeptovolt-icon.png"
              width={48}
            ></Image>
            <div>
              <h1
                className={
                  hasQuery
                    ? "text-base font-semibold leading-6 text-slate-900"
                    : "text-3xl font-semibold leading-10 text-slate-950"
                }
              >
                Zeptovolt
              </h1>
              {!hasQuery && (
                <p className="mt-1 text-sm leading-5 text-slate-500">
                  Lightning fast JLCPCB / LCSC parts search
                </p>
              )}
            </div>
          </div>
          <a
            aria-label="GitHub repository"
            className={
              hasQuery
                ? "text-slate-400 transition hover:text-slate-700"
                : "fixed right-5 top-5 text-slate-400 transition hover:text-slate-700"
            }
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
        {!hasQuery && (
          <div className="mt-4 flex flex-wrap justify-center gap-2">
            {["10k 0603", "USB connector", "STM32", "basic resistor"].map(
              (query) => (
                <button
                  key={query}
                  type="button"
                  className="rounded-full border border-slate-200 bg-white px-3 py-1.5 text-xs font-medium text-slate-600 shadow-sm transition hover:border-slate-300 hover:text-slate-900 focus:outline-none focus:ring-4 focus:ring-sky-100"
                  onClick={() => cancelLastAndSetText(query)}
                >
                  {query}
                </button>
              )
            )}
          </div>
        )}
        <div
          aria-live="polite"
          className={
            hasQuery
              ? "h-10 overflow-hidden px-1 text-sm leading-5 text-slate-500 sm:h-5"
              : "sr-only"
          }
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
                // eslint-disable-next-line @next/next/no-img-element -- LCSC blocks Next's server-side image optimizer fetches.
                <img
                  alt=""
                  className="rounded-sm"
                  decoding="async"
                  height={224}
                  loading="lazy"
                  onError={hideFailedImage}
                  src={url}
                  width={224}
                ></img>
              ) : null
            }}
          />
          {hasQuery && results.length > 0 && (
            <div className="overflow-hidden rounded-lg border border-slate-200 bg-white shadow-sm">
              <div className="overflow-x-auto">
                <table className="min-w-full table-auto">
                  <thead className="bg-slate-50 text-left text-[11px] uppercase tracking-wide text-slate-500">
                    <tr>
                      <th className="px-4 py-3 font-semibold">MPN</th>
                      <th className="px-4 py-3 font-semibold">LCSC</th>
                      <th className="px-4 py-3 font-semibold">Image</th>
                      <th className="px-4 py-3 font-semibold">Description</th>
                      <th className="px-4 py-3 text-center font-semibold">
                        Type
                      </th>
                      <th className="px-4 py-3 text-right font-semibold">
                        Price
                      </th>
                      <th className="px-4 py-3 text-right font-semibold">
                        Stock
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-100">
                    {results.map((part: Part) => (
                      <ResistorRow key={part.lcsc_id} {...part}></ResistorRow>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
          {showEmptyResults && (
            <div className="rounded-lg border border-slate-200 bg-white px-6 py-10 text-center text-sm text-slate-500 shadow-sm">
              No matching parts found.
            </div>
          )}
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
    <div className="relative w-full">
      <svg
        aria-hidden="true"
        className="pointer-events-none absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-slate-400"
        fill="none"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="2"
        viewBox="0 0 24 24"
      >
        <circle cx="11" cy="11" r="8"></circle>
        <path d="m21 21-4.3-4.3"></path>
      </svg>
      <input
        aria-label="Search parts"
        autoFocus
        value={text}
        onChange={(e) => setText(e.target.value)}
        className="h-14 w-full rounded-xl border border-slate-200 bg-white pl-12 pr-4 text-base text-slate-900 shadow-sm outline-none transition placeholder:text-slate-400 focus:border-sky-400 focus:ring-4 focus:ring-sky-100"
        placeholder="10k 0603 resistor basic"
      ></input>
    </div>
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
  server_time_us: number
}

interface SearchResponseMetadata {
  partsSearched: number
  totalResults: number
  serverTimeUs: number
  clientTimeUs: number
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

function useCancelableSWR<T>(key: string | null) {
  const controller = React.useMemo(() => createAbortController(key), [key])

  return {
    response: useSWR<TimedResponse<T>>(key, async (url: string) => {
      const startedAt = performance.now()
      const response = await fetch(url, { signal: controller.signal })
      const body = (await response.json()) as T
      const elapsedMs = Math.max(performance.now() - startedAt, 0)

      return {
        body,
        clientTimeUs: Math.round(elapsedMs * 1000),
      }
    }),
    controller,
  }
}

function createAbortController(_key: string | null): AbortController {
  return new AbortController()
}

interface TimedResponse<T> {
  body: T
  clientTimeUs: number
}
