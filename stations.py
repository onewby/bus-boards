# Generate crs.csv: requires the XML version of NaPTAN to be downloaded into this directory as "NaPTAN.xml"

from xml.etree.ElementTree import ElementTree
from defusedxml.ElementTree import parse
import pandas as pd

namespace = {"n": "http://www.naptan.org.uk/"}

records = []
tree: ElementTree = parse("NaPTAN.xml")
root = tree.getroot()
stations = root.findall(".//n:StopPoints/n:StopPoint", namespaces=namespace)
for station in stations:
    atco = station.findtext(".//n:AtcoCode", namespaces=namespace)
    crs = station.findtext(".//n:StopClassification/n:OffStreet/n:Rail/n:AnnotatedRailRef/n:CrsRef", namespaces=namespace)
    if crs is not None:
        records.append((atco, crs))
del tree

df = pd.DataFrame.from_records(data=records, columns=["ATCOCode", "CrsRef"])
df.to_csv("crs.csv", index=False)
