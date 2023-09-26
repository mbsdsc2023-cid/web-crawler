import requests
import re
from bs4 import BeautifulSoup

def analyze_html(url):
    res = requests.get(url)
    return BeautifulSoup(res.text, "html.parser")

pattern = r"MBSD\{[0-9a-zA-Z]+\}"
soup = analyze_html("https://mbsdsc2023-cid.github.io/sample-site/2023/09/26/markdown-sample.html")
target_elems = soup.find_all(string=re.compile(pattern))

for i, e in enumerate(target_elems):
    print(f"{i}: \"{e}\"");