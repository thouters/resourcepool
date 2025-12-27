

* create simple POC test on top of client/server proceses, with config file.
* separate runtime InventoryManager data from Inventory (Reuse ClientFactory as InventoryManager)

* stabilize the api for the registry so i can already add an async layer on top.

* make a (unit) test that creates two clients in async threads to request resources, both should request the same for 2
seconds, second thread with a start  delay of 1 sec. second thread should get the resource after 1sec.

* Add the HTTP layer on top. of the client API.

* make the solver generic so it can be used on a set of pools and sets of resource attributes
the unittest can then be simplified to run on strings, eg req="abc", options=["ab","abc"] => &options[1]


