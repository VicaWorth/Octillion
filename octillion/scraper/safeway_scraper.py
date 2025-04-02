import requests
import time
import random
import json
from urllib.parse import urlencode, quote_plus
import socket
import pprint

# import undetected_chromedriver as uc
# from selenium.webdriver.common.by import By
# from selenium.webdriver.support.ui import WebDriverWait
# from selenium.webdriver.support import expected_conditions as EC
# from selenium.webdriver.common.action_chains import ActionChains

STOP = False

stores = {
    "W_SACRAMENTO": 256,
}

product_search_url = (
    "https://www.safeway.com/abs/pub/xapi/pgmsearch/v1/search/products"
)

headers = {
    "accept": "application/json, text/plain, */*",
    "accept-language": "en-US,en;q=0.9",
    "dnt": "1",
    "ocp-apim-subscription-key": "5e790236c84e46338f4290aa1050cdd4",
    "priority": "u=1, i",
    "referer": "https://www.safeway.com/shop/search-results.html",
    "sec-ch-ua": '"Not(A:Brand";v="99", "Google Chrome";v="133", "Chromium";v="133"',
    "sec-ch-ua-mobile": "?0",
    "sec-ch-ua-platform": '"Windows"',
    "sec-fetch-dest": "empty",
    "sec-fetch-mode": "cors",
    "sec-fetch-site": "same-origin",
    "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
}

cookies = {
    "visid_incap_1610353": "653zL4D0TXqxbKa+6nY1Fwk212cAAAAAQUIPAAAAAADRPALlIamGYb/xSeQi9YY6",
    "incap_ses_1360_1610353": "A4IlVjG47A2AmCj2ALHfEgk212cAAAAAy0gio/y8eBhedoLhfy9jxA==",
    "akacd_PR-bg-www-prod-safeway": "3919610121~rv=75~id=76162515572b224e485e8152a397f4d4",
    "ACI_S_ECommBanner": "safeway",
    "abs_gsession": "%7B%22info%22%3A%7B%22COMMON%22%3A%7B%22Selection%22%3A%22default%22%2C%22preference%22%3A%22J4U%22%2C%22userType%22%3A%22G%22%2C%22zipcode%22%3A%2294611%22%2C%22banner%22%3A%22safeway%22%2C%22siteType%22%3A%22C%22%2C%22customerType%22%3A%22%22%2C%22resolvedBy%22%3A%22%22%7D%2C%22J4U%22%3A%7B%22zipcode%22%3A%2294611%22%2C%22storeId%22%3A%223132%22%7D%2C%22SHOP%22%3A%7B%22zipcode%22%3A%2294611%22%2C%22storeId%22%3A%223132%22%7D%7D%7D",
    "ACI_S_abs_previouslogin": "%7B%22info%22%3A%7B%22COMMON%22%3A%7B%22Selection%22%3A%22default%22%2C%22preference%22%3A%22J4U%22%2C%22userType%22%3A%22G%22%2C%22zipcode%22%3A%2294611%22%2C%22banner%22%3A%22safeway%22%2C%22siteType%22%3A%22C%22%2C%22customerType%22%3A%22%22%2C%22resolvedBy%22%3A%22%22%7D%2C%22J4U%22%3A%7B%22zipcode%22%3A%2294611%22%2C%22storeId%22%3A%223132%22%7D%2C%22SHOP%22%3A%7B%22zipcode%22%3A%2294611%22%2C%22storeId%22%3A%223132%22%7D%7D%7D",
    "SWY_SYND_USER_INFO": "%7B%22storeAddress%22%3A%22%22%2C%22storeZip%22%3A%2294611%22%2C%22storeId%22%3A%223132%22%2C%22preference%22%3A%22J4U%22%7D",
    "ACI_S_ECommSignInCount": "0",
    "SAFEWAY_MODAL_LINK": "",
    "SWY_SHARED_SESSION_INFO": "%7B%22info%22%3A%7B%22COMMON%22%3A%7B%22userType%22%3A%22G%22%2C%22zipcode%22%3A%2294611%22%2C%22banner%22%3A%22safeway%22%2C%22preference%22%3A%22J4U%22%2C%22Selection%22%3A%22default%22%2C%22wfcStoreId%22%3A%225799%22%2C%22userData%22%3A%7B%7D%2C%22grsSessionId%22%3A%227d75c50f-2923-400d-9ec5-538ce0ff2d64%22%2C%22siteType%22%3A%22C%22%2C%22customerType%22%3A%22%22%2C%22resolvedBy%22%3A%22%22%7D%2C%22J4U%22%3A%7B%22storeId%22%3A%223132%22%2C%22zipcode%22%3A%2294611%22%2C%22userData%22%3A%7B%7D%7D%2C%22SHOP%22%3A%7B%22storeId%22%3A%223132%22%2C%22zipcode%22%3A%2294611%22%2C%22userData%22%3A%7B%7D%7D%7D%7D",
    "OptanonAlertBoxClosed": "2025-03-16T20:46:47.387Z",
    "OptanonConsent": "isGpcEnabled=0&datestamp=Sun+Mar+16+2025+13%3A46%3A49+GMT-0700+(Pacific+Daylight+Time)&version=202409.1.0&browserGpcFlag=0&isIABGlobal=false&hosts=&consentId=d335b7ec-3d12-499a-a1e6-3c2286dd9d7d&interactionCount=1&isAnonUser=1&landingPath=NotLandingPage&groups=C0001%3A1%2CC0002%3A0%2CC0004%3A0%2CC0003%3A0&intType=2&geolocation=US%3BCA&AwaitingReconsent=false",
    "nlbi_1610353_2147483392": "n1poK/VR+iv/JaZo6eNT2gAAAADqt6+kKW+un720JLCTCl/L",
    "reese84": "3:i7Yz4yQN+HVEkDDR12hMhg==:2pZ+b9jjbJfgkcJC6GKo3lRxhne2DNVoo+KRLCCYA8qZ1Skoeh8Wzl9g9HU9yNFUsCpr0X1fHbO+2WCHud5RuqRsJmOCGIaoR9kVm++Atz80ghm+M2p7w6ifm5Ai3lC9oUiQaPA1ex33qRzwZ1HoipfeZALCARoCEjge3aSua9VzffPrQPjq0IM0BGQ5PmIAJJ9bYSPyXvXwOaSHy0P+vpYrhiB8sDWdhuHnXl9KXndNS/df1yoBwNpDPzgfGqaCrDeJn3595uRuZ7cdbVpdLpgieXUVq/3GrZBzM0hnygRY7LeY22tCUT82uy+vjIBEJQW5E+YLuBk9dPnwshwtjqfDbRE5XeOgNsQOOC1h/cOQG8S1nGY3QsUxqRZdW82A/AuyK827ZGlxEtunuLBY1J4FcOFEBcfVP1QSaHHEu5e/lpMQ4+JI693CFWtm00bn4wk5CYN0vSoIvZYwFdTyyt/ib976a/7bHDI3bRXNREAQVUbT3AK+ChvFCnsXjtAkUWfX9ptalxAZUs5u7QxebNXXtDBINmlR2JXvHQObyqo=:ehJfDNoZvqGibuPNeDLD2DfdVIo8hhjggbneVOAmut4=",
    "nlbi_1610353": "9eIuQbVvRV5V+JrD6eNT2gAAAABFb/IdKPoPkpczcuaPzKPj"
}

TIMEOUT = 20
RATE_LIMIT_DELAY = 2

def _get_utc_timestamp_random():
    ts = (
        str(random.randint(100, 999))
        + str(int(time.time() * 1000))
        + str(random.randint(100, 999))
    )
    return ts

"""
This code currently doesn't bypass the protections safeway has in place.
More research will need to be done, for now, I will simply be giving it cookies to use
That are copy pasted
"""
# def set_cookies():
#     """Gets cookies from Safeway.com"""
    # /abs/pub/xapi/preload/webpreload/storeflags/3132?zipcode=94611
    # https://www.safeway.com/abs/pub/xapi/preload/webpreload/storeflags/3132?zipcode=94611


def fetch_data(start, cookies, query="*"):
    global STOP

    params = {
        "request-id": _get_utc_timestamp_random(),
        "url": "https://www.safeway.com",
        "pageurl": "https://www.safeway.com",
        "pagename": "search",
        "rows": "30",
        "start": str(start),
        "search-type": "keyword",
        "storeid": stores["W_SACRAMENTO"],
        "q": query,
        "sort": "price+asc",
        "dvid": "web-4.1search",
        "channel": "instore",
        "timezone": "America/Los_Angeles",
        "visitorId": "7fdad44e-f50b-43c6-8e60-676b6ad98d73",
    }

    url = f"{product_search_url}?{urlencode(params)}"

    try:
        print("Fetching data from:", url)
        response = requests.get(url, headers=headers, cookies=cookies, timeout=TIMEOUT)
        response.raise_for_status()
        data = response.json()

        if (
            "primaryProducts" in data
            and "appCode" in data["primaryProducts"]
            and data["primaryProducts"]["appCode"] == "[GR204] [PP: 200] [SD200]"
        ):
            STOP = True
            return None

        if (
            "primaryProducts" in data
            and "appCode" in data["primaryProducts"]
            and data["primaryProducts"]["appCode"] == "400"
        ):
            raise Exception("Request-id invalid")

        return data

    except requests.exceptions.Timeout as e:
        try:
            hostname = "www.safeway.com"
            socket.gethostbyname(hostname)
        except socket.gaierror as dns_error:
            raise Exception(f"DNS resolution failed: {dns_error}") from e
        try:
            socket.create_connection((hostname, 443), timeout=5)
        except socket.error as conn_error:
            raise Exception(f"Connection to {hostname}:443 failed: {conn_error}") from e
        raise Exception(f"Request timed out after {TIMEOUT} seconds: {e}")

    except requests.exceptions.RequestException as e:
        raise Exception(f"Error fetching data from {url}: {e}")
    except Exception as e:
        raise Exception(f"Error processing data from {url}: {e}")

def parse_json(data, filename):

    filepath = f"{filename}"
    with open(filepath, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=4)
    print(f"Data saved to {filepath}")


if __name__ == "__main__":
    try:
        # cookies = set_cookies()  # Initialize cookies
        # if not cookies:
        #     print("Failed to initialize cookies. Exiting.")
        #     exit()

        # print(json.dumps(cookies, indent=4))
        print("Cookies set.")
        queries = ["*"]
        for query in queries:
            print("Starting to fetch data...")
            data = fetch_data(0, cookies, query=query)

            if data:
                if data['primaryProducts']['response']['docs']:
                    for product in data['primaryProducts']['response']['docs']:
                        filename = product['pid']
                        parse_json(data, filename)

            else:
                print("fetch_data returned None (likely STOP condition).")
            time.sleep(RATE_LIMIT_DELAY)

    except Exception as e:
        print(f"An error occurred: {e}")

    print(f"STOP flag is set to: {STOP}")