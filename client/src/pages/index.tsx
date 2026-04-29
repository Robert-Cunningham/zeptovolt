import Head from "next/head"
import { useContext, useEffect, useState } from "react"
import useSWR from "swr"
import React from "react"
import Highlighter from "react-highlight-words"
import { Tooltip } from "react-tooltip"
import { ErrorBoundary } from "react-error-boundary"

const SearchContext = React.createContext("")

const Home = () => {
  return (
    <>
      <Head>
        <title>Zeptovolt</title>
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
  const search_strings = useContext(SearchContext).split(" ")

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
        <a
          {...ttProps}
          data-tooltip-id="lcsc"
          data-tooltip-content="Click to copy"
        >
          <p
            className="text-gray-600 hover:bg-yellow-100 cursor-pointer flex-inline active:bg-green-100 transition-colors transition-none duration-100"
            onClick={() => {
              navigator.clipboard.writeText(lcsc_id)
            }}
          >
            <Highlighter searchWords={search_strings} textToHighlight={lcsc_id}>
              {" "}
            </Highlighter>
          </p>
        </a>
      </td>
      <td className="px-2 w-16 h-16">
        <a
          {...ttProps}
          data-tooltip-id="image"
          data-image-url={`https://assets.lcsc.com/images/lcsc/224x224/${image_url}`}
        >
          <img
            className="rounded-sm"
            src={`https://assets.lcsc.com/images/lcsc/96x96/${image_url}`}
          ></img>
        </a>
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
  process.env.NODE_ENV === "development"
    ? "http://localhost:8090"
    : "https://api.zeptovolt.com"

const CentralColumn = () => {
  const [text, setText] = useState<string>("")
  const [results, setResults] = useState<Part[]>([])
  const [startTime, setStartTime] = useState<
    Record<string, number | undefined>
  >({})
  const [endTime, setEndTime] = useState<Record<string, number | undefined>>({})

  const dbText = useDebounce(text, 150)

  const { response, controller } = useCancelableSWR<Part[]>(
    `${API_ENDPOINT}/search?${new URLSearchParams({ q: dbText })}`
  )
  const { data, isLoading, error } = response

  useEffect(() => {
    if (data && !isLoading && !error && data !== results) {
      setResults(data)
      if (!endTime[text]) {
        setEndTime((endTime) => ({ ...endTime, [text]: Date.now() }))
      }
    }
  }, [data, setEndTime, setResults, text])

  const cancelLastAndSetText = (newText: string) => {
    controller.abort()
    setStartTime((startTime) => ({ ...startTime, [newText]: Date.now() }))
    setEndTime((endTime) => ({ ...endTime, [text]: undefined }))
    setText(newText)
  }

  const time =
    endTime[text] && startTime[text] && endTime[text]! - startTime[text]!

  return (
    <div className="md:max-w-4xl mx-auto flex flex-col gap-4">
      <SearchContext.Provider value={text}>
        <p className="text-gray-800 font-semibold mx-auto text-md">
          Fast JLCPCB Parts Search
        </p>
        <SearchBox {...{ text, setText: cancelLastAndSetText }}></SearchBox>
        {text && time && (
          <p className="text-gray-400 text-sm pl-3">
            Searched 287k JLCPCB parts in {time}ms.{" "}
            {results.length === 100
              ? "Showing first 100 results."
              : `${results.length} results.`}
          </p>
        )}
        <ErrorBoundary FallbackComponent={ErrorFallback}>
          <Tooltip id="stock" {...ttElProps} />
          <Tooltip id="price" {...ttElProps} />
          <Tooltip id="bore" {...ttElProps} />
          <Tooltip id="lcsc" {...ttElProps} />
          <Tooltip
            id="image"
            {...ttElProps}
            render={({ content, activeAnchor }) => {
              const url = activeAnchor?.getAttribute("data-image-url")
              return <img className="rounded-sm" src={url!}></img>
            }}
          />
          <table className="table-auto rounded-md">
            <tbody>
              {results.map((part: Part, i: number) => (
                <ResistorRow
                  key={part.manufacturer_id + part.description + part.price}
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
  image_url: string
  datasheet_url: string
  basic_or_extended: string
  price: number
  stock: number
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
    response: useSWR<T>(key, (url: string) =>
      fetch(url, { signal: controller.signal }).then(
        (x) => x.json() as Promise<T>
      )
    ),
    controller,
  }
}
