import requests
import time
import random
import json
from urllib.parse import urlencode, quote_plus
import socket

STOP = False
print(f"DEBUG: Initial STOP value: {STOP}")

stores = {
    "W_SACRAMENTO": 256,  # This will be overridden by the headers.
}
print(f"DEBUG: Stores: {stores}")

product_search_url = (
    "https://www.safeway.com/abs/pub/xapi/pgmsearch/v1/search/products"
)
print(f"DEBUG: Product search URL: {product_search_url}")

# Headers from the curl command (CRITICAL)
headers = {
    "accept": "application/json, text/plain, */*",
    "accept-language": "en-US,en;q=0.9",
    "dnt": "1",
    "ocp-apim-subscription-key": "5e790236c84e46338f4290aa1050cdd4",
    "priority": "u=1, i",
    "referer": "https://www.safeway.com/shop/search-results.html?q=apples",
    "sec-ch-ua": '"Not(A:Brand";v="99", "Google Chrome";v="133", "Chromium";v="133"',  # Use the exact string
    "sec-ch-ua-mobile": "?0",
    "sec-ch-ua-platform": '"Windows"',  # Use the exact string
    "sec-fetch-dest": "empty",
    "sec-fetch-mode": "cors",
    "sec-fetch-site": "same-origin",
    "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
    # Cookies will be handled separately
}
print(f"DEBUG: Headers: {headers}")

cookies = {
    "SAFEWAY_MODAL_LINK": "",
    "SWY_SHARED_SESSION_INFO": "%7B%22info%22%3A%7B%22COMMON%22%3A%7B%22userType%22%3A%22G%22%2C%22zipcode%22%3A%2294611%22%2C%22banner%22%3A%22safeway%22%2C%22preference%22%3A%22J4U%22%2C%22Selection%22%3A%22default%22%2C%22wfcStoreId%22%3A%225799%22%2C%22userData%22%3A%7B%7D%2C%22grsSessionId%22%3A%22feba2f2c-0b78-421d-b938-ef60feb50b10%22%2C%22siteType%22%3A%22C%22%2C%22customerType%22%3A%22%22%2C%22resolvedBy%22%3A%22%22%7D%2C%22J4U%22%3A%7B%22storeId%22%3A%223132%22%2C%22zipcode%22%3A%2294611%22%2C%22userData%22%3A%7B%7D%7D%2C%22SHOP%22%3A%7B%22storeId%22%3A%223132%22%2C%22zipcode%22%3A%2294611%22%2C%22userData%22%3A%7B%7D%7D%7D",
    "SWY_SYND_USER_INFO": "%7B%22storeAddress%22%3A%22%22%2C%22storeZip%22%3A%2294611%22%2C%22storeId%22%3A%223132%22%2C%22preference%22%3A%22J4U%22%7D",
    "ACI_S_ECommSignInCount": "0",
    "OptanonAlertBoxClosed": "2025-03-05T20:35:19.698Z",
    "akacd_PR-bg-www-prod-safeway": "3918850421~rv=37~id=60bdc74d85cf8b9a73417d920e141c5b",
    "reese84": "3:93D9IATT1qQ3oxG9b3cVDg==:rHMcx80VOaUoVJhIZCDSa/BY/C+xt4jjVqc0V05uiUSl7bbvruo0UUIlGwHdUmdhvpQ72ZeW8ts8wTrNWLI+4I8a/ZLHXoPqlAZvBui2XN+nb9ReCh2Z1OiOBdB6zSAdnoknHMAHZn+thFYV+KNZeoqk14bfQtZ4CGpiLMSF8SnEkvS5tjeK4vYoTsMrsOA1dl61epvbXIheCQkz/EvCImeZXATJoVGzWgrVwOBGMmhYWN5z7k+vQERrkiOnHLkPga4Nj3kCRBChYkg6sOlVi4x4GoyyJIqCMctgDTE1tg1dKXNcQJrpbr6d3dlTY1PXIeIOLNqJZpOFJo9IQQlCTXTUrZVqLi7NVapNmZxhGJH3jwa0kl8yR0CpzYR3jDkg45Hay4IcQ2sikcaWfECB1XlcsggrGx1YvgtcQHOMKrKa0dGcr0oSc9Fs4hQyNdZ6rVVEJHzhdxrSVCYlDSbYkOxyfOT+SVXO6nJImqkRQY4UhXr/DSkT0RUaXvdQFTz47eb/DPPqKFCqDWzROYdYZQ==:nYlaJjbNu0fpbEQdFxaHBGwTQ+mBdRCzqCM/bFPgrto=",
    "salsify_session_id": "75fa7fc4-7f97-4a7c-b735-bd587d5f3b02",
    "absVisitorId": "7fdad44e-f50b-43c6-8e60-676b6ad98d73",
    "incap_ses_170_1610353": "GfjVbidg9Ff40mxFZvZbAnWey2cAAAAAaqz9juaf1E34L08Bnl7Mtg==",
    "__eoi": "ID=4ef261fb5a27048c:T=1741203384:RT=1741397624:S=AA-AfjYi7GBmwiA-rV3Xs4c_nvzA",
    "mbox": "PC#c445a48cdbe74defac17b34ec97dac9b.35_0#1804642434|session#df8f75a86dd548879f9f850899c6751c#1741399494",
    "nlbi_1610353": "fvVqDpfQ+UvaqQ2I6eNT2gAAAAB5n964PO8hOa54kfy5LlRM",
    "OptanonConsent": "isGpcEnabled=0&datestamp=Fri+Mar+07+2025+17%3A33%3A44+GMT-0800+(Pacific+Standard+Time)&version=202409.1.0&browserGpcFlag=0&isIABGlobal=false&hosts=&consentId=bcd1b7b8-a289-45d4-837b-df533c838dd7&interactionCount=2&isAnonUser=1&landingPath=NotLandingPage&groups=C0001%3A1%2CC0002%3A0%2CC0004%3A0%2CC0003%3A1&AwaitingReconsent=false&intType=3&geolocation=US%3BCA",
    "visid_incap_1610353": "jejKaDXISsqliMVvRRSCprOnyGcAAAAAQUIPAAAAAABUy4SlCjnP8lyIeaowFGFr",
    "nlbi_1610353_2147483392": "msWlHnoNNwhyKEMX6eNT2gAAAAAqbbF9JcXPsBqFZoMi9xUA",
    "ACI_S_ECommBanner": "safeway",
    "at_check": "true"
}
print(f"DEBUG: Cookies: {cookies}")

TIMEOUT = 20  # seconds.  Increased for testing.
RATE_LIMIT_DELAY = 2  # Seconds.  For rate limiting.

def _get_utc_timestamp_random():
    ts = (
        str(random.randint(100, 999))
        + str(int(time.time() * 1000))
        + str(random.randint(100, 999))
    )
    print(f"DEBUG: Generated timestamp: {ts}")
    return ts


def fetch_data(start, query="*"):
    global STOP
    print(f"DEBUG: fetch_data called with start={start}, query={query}")

    # Construct parameters, mirroring the curl command
    params = {
        "request-id": _get_utc_timestamp_random(),  # Keep generating a new ID
        "url": "https://www.safeway.com",
        "pageurl": "https://www.safeway.com",
        "pagename": "search",
        "rows": "30",
        "start": str(start),  # Ensure start is a string
        "search-type": "keyword",
        "storeid": stores["W_SACRAMENTO"],
        # "featured"
        "q": query,
        "sort": "",
        "dvid": "web-4.1search",
        "channel": "instore",
        # "wineshopstoreid": "5799",  # From the curl command
        "timezone": "America/Los_Angeles",  # From the curl command
        # "zipcode": "94611",  # From the curl command
        "visitorId": "7fdad44e-f50b-43c6-8e60-676b6ad98d73",  # From the curl command
        # "pgm": "intg-search,wineshop,merch-banner",  # From the curl command
        # "banner": "safeway",  # From the curl command
        # "variantTile": "ACIP282538_a",  # From the curl command
    }
    print(f"DEBUG: Request parameters: {params}")

    url = f"{product_search_url}?{urlencode(params)}"
    print(f"DEBUG: Constructed URL: {url}")

    try:
        print(f"DEBUG: Sending request with timeout={TIMEOUT} seconds...")
        # Include both headers AND cookies
        response = requests.get(url, headers=headers, cookies=cookies, timeout=TIMEOUT)
        print(f"DEBUG: Response received. Status code: {response.status_code}")
        response.raise_for_status()
        print(f"DEBUG: Response passed status check.")
        data = response.json()
        print(f"DEBUG: Response parsed as JSON.")

        if (
            "primaryProducts" in data
            and "appCode" in data["primaryProducts"]
            and data["primaryProducts"]["appCode"] == "[GR204] [PP: 200] [SD200]"
        ):
            STOP = True
            print(f"DEBUG: STOP condition met. Setting STOP to {STOP}")
            return None

        if (
            "primaryProducts" in data
            and "appCode" in data["primaryProducts"]
            and data["primaryProducts"]["appCode"] == "400"
        ):
            print(f"DEBUG: Invalid request-id detected.")
            raise Exception("Request-id invalid")


        print(f"DEBUG: Returning data.")
        return data

    except requests.exceptions.Timeout as e:
        print(f"DEBUG: Request timed out after {TIMEOUT} seconds.")
        print(f"DEBUG: Timeout Exception Details: {e}")

        # Check for DNS resolution issues
        try:
            hostname = "www.safeway.com"
            print(f"DEBUG: Attempting to resolve hostname: {hostname}")
            ip_address = socket.gethostbyname(hostname)
            print(f"DEBUG: Hostname resolved to IP address: {ip_address}")
        except socket.gaierror as dns_error:
            print(f"DEBUG: DNS resolution failed: {dns_error}")
            raise Exception(f"DNS resolution failed: {dns_error}") from e

        # Check for basic connectivity issues
        try:
            print(f"DEBUG: Attempting to connect to {hostname} on port 443...")
            socket.create_connection((hostname, 443), timeout=5)
            print(f"DEBUG: Connection to {hostname}:443 successful.")
        except socket.error as conn_error:
            print(f"DEBUG: Connection to {hostname}:443 failed: {conn_error}")
            raise Exception(f"Connection to {hostname}:443 failed: {conn_error}") from e

        raise Exception(f"Request timed out after {TIMEOUT} seconds: {e}")

    except requests.exceptions.RequestException as e:
        print(f"DEBUG: RequestException caught: {e}")
        raise Exception(f"Error fetching data from {url}: {e}")
    except Exception as e:
        print(f"DEBUG: Other Exception caught: {e}")
        raise Exception(f"Error processing data from {url}: {e}")

def parse_json(data):
    if data['primaryProducts']['response']['docs']:
        for product in data['primaryProducts']['response']['docs']:
            filename = product['pid']
            filepath = f"{filename}" 

            with open(filepath, 'w', encoding='utf-8') as f:
                json.dump(data, f, indent=4)  

            print(f"Data saved to {filepath}")

if __name__ == "__main__":
    print("DEBUG: Starting main execution...")
    try:
        queries = ["oranges", "cookies", "avocados", "cups"]  # Example: Multiple queries
        for query in queries:
            print(f"DEBUG: Processing query: {query}")
            data = fetch_data(0, query=query)  # start=0 for each new query

            if data:
                parse_json(data)

                print("DEBUG: Data fetched successfully.  Printing a summarized version:")
                print("DEBUG: Saving data to json file with ")
            else:
                print("DEBUG: fetch_data returned None (likely STOP condition).")

            time.sleep(RATE_LIMIT_DELAY)  # Rate limiting

    except Exception as e:
        print(f"DEBUG: An error occurred in main: {e}")

    print(f"DEBUG: STOP flag is set to: {STOP}")
    print("DEBUG: End of main execution.")