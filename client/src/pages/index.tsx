import Head from 'next/head'
import Image from 'next/image'
import { Inter } from '@next/font/google'
import styles from '@/styles/Home.module.css'
import { useContext, useEffect, useState } from 'react'
import useSWR from 'swr'
import React from 'react'
import Highlighter from 'react-highlight-words'
import { Tooltip } from 'react-tooltip'
import { renderToHTML } from 'next/dist/server/render'

const SearchContext = React.createContext("");

const Home = () => {
  return (
    <>
      <Head>
        <title>Zeptovolt</title>
      </Head>
      <main>
        <div className="py-8 bg-slate-100 h-screen w-screen">
          <CentralColumn></CentralColumn>
        </div>
      </main>
    </>
  )
}

/*
Can you write me a typescript function which converts a float to a string of US dollars? For example, the price 3.34234 should be displayed as $3.34. Any prices less than a cent should be displayed to one significant digit. For example, 0.000000234 should be displayed as $0.0000002.
*/

function formatUSDPrice(value: number): string {
  const options: Intl.NumberFormatOptions = {
    style: 'currency',
    currency: 'USD',
  };

  // Check if the value is less than a cent
  if (value < 0.01) {
    // Calculate the number of significant digits after the decimal point
    const significantDigits = Math.ceil(-Math.log10(value));

    // Set the minimum and maximum fraction digits to the significant digits
    options.minimumFractionDigits = significantDigits;
    options.maximumFractionDigits = significantDigits;
  } else {
    // Set the minimum and maximum fraction digits to 2 (for cents)
    options.minimumFractionDigits = 2;
    options.maximumFractionDigits = 2;
  }

  // Format the value using Intl.NumberFormat
  const formatter = new Intl.NumberFormat('en-US', options);
  return formatter.format(value);
}

function shortenNumber(num: number): string | number {
  if (typeof num !== "number" || isNaN(num)) {
    return num;
  }

  let suffix = "";
  let divisor = 1;

  if (num >= 1_000_000) {
    suffix = "M";
    divisor = 1_000_000;
  } else if (num >= 1_000) {
    suffix = "k";
    divisor = 1_000;
  }

  const shortNum = Math.round(num / divisor);
  return shortNum.toString() + suffix;
}

// Example usage:
// console.log(shortenNumber(4711));     // "4k"
// console.log(shortenNumber(5125829));   // "5M"
// console.log(shortenNumber(NaN));       // NaN


function ResistorRow({ image_url, description, manufacturer_id, price, stock, basic_or_extended, datasheet_url }: Part) {
  const search_strings = useContext(SearchContext).split(" ")

  return (
    <tr className="bg-white border py-2">
      <td className="px-2">
        <a href={datasheet_url}>
          <p className="text-gray-600">
            <Highlighter searchWords={search_strings} textToHighlight={manufacturer_id}> </Highlighter>
          </p>
        </a>
      </td>
      <td className="px-2 w-16 h-16">
        <a {...ttProps} data-tooltip-id="image" data-image-url={`https://assets.lcsc.com/images/lcsc/224x224/${image_url}`}>
          <img className="rounded-sm" src={`https://assets.lcsc.com/images/lcsc/96x96/${image_url}`} ></img>
        </a>
      </td>
      <td className="px-2">
        <p className="text-gray-600">
          <Highlighter searchWords={search_strings} textToHighlight={description}> </Highlighter>
        </p>
      </td>
      <td className="px-2">
        <p className="text-gray-600">
          <a {...ttProps} data-tooltip-id="bore" data-tooltip-content={basic_or_extended.toLowerCase() + " part type"}>{basic_or_extended.at(0)?.toUpperCase()}</a>
        </p>
      </td>
      <td className="px-2">
        <p className="text-gray-600">
          <a {...ttProps} data-tooltip-id="price" data-tooltip-content={'$' + price}>{formatUSDPrice(price)}</a>
        </p>
      </td>
      <td className="px-2">
        <p className="text-gray-600"><a {...ttProps} data-tooltip-id="stock" data-tooltip-content={stock + " in stock"}>{shortenNumber(stock)}</a></p>
      </td>
    </tr>
  );
}

const ttProps = {
  "data-tooltip-delay-show": 100,
  className: "cursor-pointer",
}

const ttElProps = {
  style: { backgroundColor: "rgb(0, 0, 0)", opacity: 1 }
}

const API_ENDPOINT = process.env.NODE_ENV === "development" ? "http://localhost:8090" : "https://api.zeptovolt.com"

const CentralColumn = () => {
  const [text, setText] = useState<string>("")
  const [results, setResults] = useState<Part[]>([]);

  const dbText = useDebounce(text, 150);

  const { response, controller } = useCancelableSWR(`${API_ENDPOINT}/search?q=${dbText}`)
  const { data, isLoading, error } = response;

  useEffect(() => {
    if (data && !isLoading && !error) {
      setResults(data)
    }
  }, [data])

  const cancelLastAndSetText = (newText: string) => {
    controller.abort()
    setText(newText)
  }

  return <div className="md:max-w-4xl mx-auto flex flex-col gap-4">
    <SearchContext.Provider value={text}>
      <SearchBox {...{ text, setText: cancelLastAndSetText }}></SearchBox>
      <Tooltip id="stock" {...ttElProps} />
      <Tooltip id="price"{...ttElProps} />
      <Tooltip id="bore"{...ttElProps} />
      <Tooltip id="image" {...ttElProps} render={({ content, activeAnchor }) => {
        const url = activeAnchor?.getAttribute("data-image-url");
        return (
          <img className="rounded-sm" src={url!}></img>
        )
      }} />
      <table className="table-auto">
        <tbody>
          {results.map((part: Part) => (
            <ResistorRow key={part.manufacturer_id + part.description + part.price} {...part}></ResistorRow>
          ))}
        </tbody>
      </table>
    </SearchContext.Provider>
  </div>
}

// <input type="search" id="default-search" className="block w-full p-4 pl-10 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="Search Mockups, Logos..." required

const SearchBox = ({ text, setText }: { text: string, setText: (a0: string) => void }) => {
  return <input value={text} onChange={e => setText(e.target.value)} className="w-full h-8 rounded-lg text-sm p-5 border" placeholder="10k 0603 resistor"></input>
}

interface Part {
  description: string,
  lscs_id: string,
  manufacturer_id: string,
  image_url: string,
  datasheet_url: string,
  basic_or_extended: string,
  price: number,
  stock: number
}

export default Home;

// Hook
// T is a generic type for value parameter, our case this will be string
function useDebounce<T>(value: T, delay: number): T {
  // State and setters for debounced value
  const [debouncedValue, setDebouncedValue] = useState<T>(value);
  useEffect(
    () => {
      // Update debounced value after delay
      const handler = setTimeout(() => {
        setDebouncedValue(value);
      }, delay);
      // Cancel the timeout if value changes (also on delay change or unmount)
      // This is how we prevent debounced value from updating if value is changed ...
      // .. within the delay period. Timeout gets cleared and restarted.
      return () => {
        clearTimeout(handler);
      };
    },
    [value, delay] // Only re-call effect if value or delay changes
  );
  return debouncedValue;
}


// @ts-ignore
const fetcher = (...args) => fetch(...args).then(x => x.json())

//@ts-ignore
function useCancelableSWR(key) {
  const controller = new AbortController()
  return { response: useSWR(key, (url: string) => fetch(url, { signal: controller.signal }).then(x => x.json())), controller }
}