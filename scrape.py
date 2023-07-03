import requests
from bs4 import BeautifulSoup
import json

def scrape(url, filename, five_star, four_star, default, chara):
    response = requests.get(url)
    soup = BeautifulSoup(response.text, 'html.parser')
    
    table = soup.find('table', {'class': 'article-table'})

    data = []

    for row in table.find_all('tr')[1:]:
        cells = row.find_all('td')

        item_link = cells[1].find('a')['href']
        item_response = requests.get("https://genshin-impact.fandom.com" + item_link)
        item_soup = BeautifulSoup(item_response.text, 'html.parser')

        description_div = item_soup.find('div', class_="description-content")
        description = description_div.text.strip() if description_div else None
        try:
            if chara:
                image_url = item_soup.find("a", {"title": "Wish"}).img["src"].split("revision")[0]
            else:
                image_url = item_soup.find("a", {"title": "Base"}).img["src"].split("revision")[0]
        except:
            print(f"Failed to get Image: {item_link}")
            image_url = None
        if chara:
            l = "https://genshin-impact.fandom.com/" + item_link.split('/wiki/')[1] + '/Voice-Overs'
            print(l)
            vo_response = requests.get(l)
            vo_soup = BeautifulSoup(vo_response.text, 'html.parser')
            try:
                hello_voiceline = vo_soup.find("th", {"id": "Hello"}).findNextSibling().find("span", {"lang": "en"}).text
            except AttributeError: #Cyno
                try:
                    hello_voiceline = vo_soup.find("th", {"id": "Hello:_The_Present"}).findNextSibling().find("span", {"lang": "en"}).text
                except AttributeError: #Heizou
                    try:
                        hello_voiceline = vo_soup.find("th", {"id": "Hello..."}).findNextSibling().find("span", {"lang": "en"}).text
                    except: 
                        print(f"Failed to get HELLO: {item_link}")
                        hello_voiceline = None
                    
        else:
            hello_voiceline = None


        name = cells[1].text.strip()

        img_tag = cells[2].find('img')
        if img_tag:
            alt_text = img_tag['alt']

            if "5" in alt_text:
                item_type = five_star
            elif "4" in alt_text:
                item_type = four_star
            else:
                item_type = default

            item = {"name": name, "item_type": item_type, "description": description if description else hello_voiceline, "image_url": image_url}
            data.append(item)

    with open(filename, 'w') as f:
        json.dump(data, f, indent=4)

characters_url = "https://genshin-impact.fandom.com/wiki/Character/List"
weapons_url = "https://genshin-impact.fandom.com/wiki/Weapon/List"

scrape(characters_url, 'genshin_characters.json', "FiveStarCharacterLimited", "FourStarCharacter", "FourStarCharacter", True)
scrape(weapons_url, 'genshin_weapons.json', "FiveStarWeapon", "FourStarWeapon", "ThreeStarItem", False)

print("CHECK FOR NULL VALUES")
