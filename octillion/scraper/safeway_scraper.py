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
    "OptanonAlertBoxClosed": "2025-03-05T20:35:19.698Z",
    "OptanonConsent": "isGpcEnabled=0&datestamp=Mon+Mar+10+2025+20%3A16%3A09+GMT-0700+(Pacific+Daylight+Time)&version=202409.1.0&browserGpcFlag=0&isIABGlobal=false&hosts=&consentId=bcd1b7b8-a289-45d4-837b-df533c838dd7&interactionCount=2&isAnonUser=1&landingPath=NotLandingPage&groups=C0001%3A1%2CC0002%3A0%2CC0004%3A0%2CC0003%3A1&AwaitingReconsent=false&intType=3&geolocation=US%3BCA",
    "SWY_SHARED_SESSION_INFO": "%7B%22info%22%3A%7B%22COMMON%22%3A%7B%22userType%22%3A%22G%22%2C%22zipcode%22%3A%2294611%22%2C%22banner%22%3A%22safeway%22%2C%22preference%22%3A%22J4U%22%2C%22Selection%22%3A%22default%22%2C%22wfcStoreId%22%3A%225799%22%2C%22userData%22%3A%7B%7D%2C%22grsSessionId%22%3A%22e68f9e59-fd35-46a5-811a-e50a8d4bfd10%22%2C%22siteType%22%3A%22C%22%2C%22customerType%22%3A%22%22%2C%22resolvedBy%22%3A%22%22%7D%2C%22J4U%22%3A%7B%22storeId%22%3A%223132%22%2C%22zipcode%22%3A%2294611%22%2C%22userData%22%3A%7B%7D%7D%2C%22SHOP%22%3A%7B%22storeId%22%3A%223132%22%2C%22zipcode%22%3A%2294611%22%2C%22userData%22%3A%7B%7D%7D%7D%7D",
    "SWY_SYND_USER_INFO": "%7B%22storeAddress%22%3A%22%22%2C%22storeZip%22%3A%2294611%22%2C%22storeId%22%3A%223132%22%2C%22preference%22%3A%22J4U%22%7D",
    "__eoi": "ID=4ef261fb5a27048c:T=1741203384:RT=1741662963:S=AA-AfjYi7GBmwiA-rV3Xs4c_nvzA",
    "absVisitorId": "7fdad44e-f50b-43c6-8e60-676b6ad98d73",
    "mbox": "PC#c445a48cdbe74defac17b34ec97dac9b.35_0#1804907799|session#22b22cf095ff40b6aaa8b3a8b2383f18#1741664859",
    "reese84": "3:yc/MhB9jK33vOPi5t45a7Q==:my9r5alF4nGVnnMWlvbt4ZfL9vBpy5duHhLdcoSf6SP8dBSWA5PlbSXAYMUudhgIMc/LthkbvGzMHpllLU+/HLY8Kb5ULhLX9FR+iaSboJz2MPsOgKpRhlO0xb/MuKO/txEDdbhUclGo5R/Hay70zY2dSNHJoHUZjICx274PANsq8bK7/k6tcDOAvBhuPMJf2C9eSNx5yWOG/Jeh0ndj4mcIAEDaHbJrSjQcKdPEYWHcJIIWxUW4o13NNc85TdIQZAT44qtbM5AeHl0fLgBHndLlmNHASHc4FNr94DNhlVTM+4ereZQi9FfcCnpf8ID/zFp6vC3JtCLMcixRM3mFzcVOwbPZdnAtD0cSBWj6LpO44cEgDAK26oOqdfGMq0MvvsNuAaT+R3bc1ON/me3mkVsIpwgEzqlhub6+llq16o9GxIqlvJneLt/f58R5kd/p8Y9cR4ljbVa0AsOKmhZ41GcMqsVq+NkV55MsEILsE8BjY23V5WG+teLsnQkmf6VMMjmWMV66ZQWmEpaG4FrUgQ==:sRR33PsepS9jsTVViVEnCdqeRwuO41yYSYV8aN94pME=",
    "salsify_session_id": "75fa7fc4-7f97-4a7c-b735-bd587d5f3b02",
    "visid_incap_1610353": "jejKaDXISsqliMVvRRSCprOnyGcAAAAAQUIPAAAAAABUy4SlCjnP8lyIeaowFGFr"   
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
def set_cookies():
    """Gets ALL cookies from Safeway using undetected_chromedriver, with anti-detection measures."""

    options = uc.ChromeOptions()
    options.add_argument("--disable-infobars")
    options.add_argument("--window-size=1920,1080")
    options.set_capability("goog:chromeOptions", {"args": [f"user-agent={headers['user-agent']}"]})
    options.add_argument(f"--lang={headers['accept-language']}")
    
    if "--headless" in options.arguments:
        options.add_argument("--disable-gpu")
        
    for header, value in headers.items():
        if header.lower() not in ("user-agent", "sec-ch-ua", "sec-ch-ua-mobile", "sec-ch-ua-platform"):
            # We already handled user-agent above.  The sec-ch-* headers are best set via capabilities.
            if isinstance(value, str): 
                options.add_argument(f"--{header}={value}") # add it to the options
            else:
                print("Value not string")
                
    driver = uc.Chrome(options=options, use_subprocess=True)

    try:
        driver.get("https://www.safeway.com/")
        def random_delay(min_sec=0.5, max_sec=5.5):
            time.sleep(random.uniform(min_sec, max_sec))

        random_delay()

        print("HERE")
        print(driver.get_cookies())
        print("BEFORE SCROLL")
        try:
            driver.execute_script("window.scrollTo(0, document.body.scrollHeight/4);")
            random_delay()
            driver.execute_script("window.scrollTo(0, document.body.scrollHeight/2);")
            random_delay()
            driver.execute_script("window.scrollTo(0, 0);")
            random_delay()
        except:
             print("Error while initial scrolling")

        try:
            logo = driver.find_element(By.XPATH, "//a[@aria-label='safeway.com']")
            search_bar = driver.find_element(By.ID, "search-input-id")

            actions = ActionChains(driver)
            actions.move_to_element(logo).perform()
            random_delay(0.2, 0.8)
            actions.move_to_element(search_bar).perform()
            random_delay(0.2, 0.8)
            actions.move_by_offset(random.randint(100, 500), random.randint(100, 300)).perform()
            random_delay()

        except Exception as e:
            print("Error trying to find elements to move the mouse to ", e)
            actions = ActionChains(driver)
            try:
                actions.move_by_offset(random.randint(100, 500), random.randint(100, 300)).perform()
                random_delay()
            except:
                print("Moving the mouse failed")
                pass

        try:
            WebDriverWait(driver, 30).until(
                EC.presence_of_element_located((By.ID, "onetrust-accept-btn-handler"))
            ).click()
            print("Accepted cookies via banner.")
        except Exception as e:
            print(f"Cookie banner interaction failed, or banner didn't appear: {e}")

        random_delay(3, 6)
        
        
        cookies = driver.get_cookies()
        cookie_dict = {cookie['name']: cookie['value'] for cookie in cookies}
        return cookie_dict

    except Exception as e:
        print(f"Error getting cookies with Selenium: {e}")
        return {}
    finally:
        driver.quit()

def fetch_data(start, cookies, query="*"):
    global STOP

    params = {
        "request-id": _get_utc_timestamp_random(),
        "url": "https://www.safeway.com",
        "pageurl": "https://www.safeway.com",
        "pagename": "search",
        "rows": "100",
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

        queries = ["oranges", "cookies"]
        for query in queries:
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