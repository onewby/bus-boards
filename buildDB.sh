#!/usr/bin/env sh

echo "Downloading NPTG"
curl -X 'GET' \
  'https://naptan.api.dft.gov.uk/v1/nptg' \
  -H 'accept: */*' \
  --output './NPTG.xml'
echo "Downloading CSV NaPTAN"
curl -X 'GET' \
  'https://naptan.api.dft.gov.uk/v1/access-nodes?dataFormat=csv' \
  -H 'accept: */*' \
  --output './Stops.csv'
python3 stance_grouping.py
./rebuildDB.sh