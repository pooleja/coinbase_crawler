# Coinbase Crawler

Calculate gains/losses on Coinbase Pro (GDAX) trades.

## Daily Prices

Bitstamp APIs are used to get OHLC prices. This will be used for the cost basis of any deposits.

Example of json API:
https://www.bitstamp.net/api/v2/ohlc/btcusd/?step=86400&limit=365&start=1451606400

You can get different year prices by using a different start timestamp. E.g. 1451606400 is Jan 1 2016.

## Cost Basis Override

TODO: Create a file that allows manually setting cost basis for a specific deposit.

## Trade Data Report

Export the Coinbase Pro report from your trades in CSV mode and put them into the "trades" folder.
