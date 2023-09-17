import requests

def fetch_data_from_coingecko(coin_id):
    url = f"https://api.coingecko.com/api/v3/coins/{coin_id}/market_chart?vs_currency=usd&days=15&interval=daily"
    
    response = requests.get(url)
    response.raise_for_status()  # 如果请求不成功，会引发HTTPError异常
    
    data = response.json()
    return data

def calculate_average_of_prices(data):
    prices = data.get("prices", [])
    
    if not prices:
        return None
    
    total = sum(price[1] for price in prices)
    average = total / len(prices)
    
    return average

def main():
    coin_ids = ['matic-network', 'avalanche-2', 'ethereum', 'binancecoin']
    
    for coin_id in coin_ids:
        try:
            data = fetch_data_from_coingecko(coin_id)
            average_price = calculate_average_of_prices(data)
            
            if average_price is not None:
                print(f"The average price of {coin_id} over the past 15 days is: ${average_price:.2f}")
            else:
                print(f"No price data found for {coin_id}.")
        
        except requests.RequestException as e:
            print(f"Error fetching data for {coin_id}: {e}")

if __name__ == "__main__":
    main()

