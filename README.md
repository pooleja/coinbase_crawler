# Coinbase Crawler

Calculate gains/losses on Coinbase Pro (GDAX) trades.

## Daily Prices

Bitstamp APIs are used to get OHLC prices. This will be used for the cost basis of any deposits.

Example of json API:
https://www.bitstamp.net/api/v2/ohlc/btcusd/?step=86400&limit=365&start=1451606400

You can get different year prices by using a different start timestamp. E.g. 1451606400 is Jan 1 2016.

## Cost Basis Override

To override the daily price of a deposit for cost basis reasons, you can list your deposit transfer IDs plus total USD cost basis. Instead of using the daily price to calculate the cost basis for that deposit it will simply just use the amount you specify in the csv. See `/deposits/deposits.example.csv` for an example. Your file should be named `deposits.csv` in that folder.

## Different Assets

TODO: Support assets other than BTC

## Trade Data Report

Export the Coinbase Pro report from your trades in CSV mode and put them into the "trades" folder. See `/trades/trades.example.csv` for an example of the format. You should export from the beginning of time (e.g. 2015) so that you can pull in all deposits and withdraws.

The code will then iterate over all items in the trades.csv file and process in FIFO mode.

All deposits of crypto will be added to a "known lots" list using the daily bitstamp price as the cost basis. If a cost basis override is specified that will be used instead.

Withdraws of crypto will remove the oldest known lots adding up to the withdraw amount.

Buys will add new known lots using the price purchased at.

Sells will calculate the long term or short term gain or loss using the oldest known lots.

Fees and Rebates will be added up per year.

Conversions (e.g. USD to USDC) will be ignored for tax purposes.

For each year, the code will write out a csv with all sell actions, including cost bases and gains and losses, plus fees. It will also print out a summary for each year aggregating all values to provide the total amounts to be used in tax reporting.
