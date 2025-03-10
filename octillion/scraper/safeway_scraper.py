import requests
import time
import random
import json
from urllib.parse import urlencode, quote_plus
import socket
import pprint

import undetected_chromedriver as uc
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.common.action_chains import ActionChains

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
    "referer": "https://www.safeway.com/shop/search-results.html?q=apples",
    "sec-ch-ua": '"Not(A:Brand";v="99", "Google Chrome";v="133", "Chromium";v="133"',
    "sec-ch-ua-mobile": "?0",
    "sec-ch-ua-platform": '"Windows"',
    "sec-fetch-dest": "empty",
    "sec-fetch-mode": "cors",
    "sec-fetch-site": "same-origin",
    "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
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
        "rows": "30",
        "start": str(start),
        "search-type": "keyword",
        "storeid": stores["W_SACRAMENTO"],
        "q": query,
        "sort": "",
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