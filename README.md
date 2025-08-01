# Offline Video Delivery System

[AID India](https://aidindia.in/) helps to bring education and educational resources to children in remote villages of India.
These villages often face challenges with the stability of their internet connections. In such an environment,
serving quality lectures can be challenging due to constraints of bandwidth, cost and latency.

The `Offline Video Delivery System` (`VDS`) is the result of a collaboration between AID India and T4EQ in an
attempt to bring quality educational videos to these children by caching the content locally at the
point where the data is served.

The `VDS` system consists of:
- A frontend video player
- A backend serving locally cached video content to devices in a local network.
- A management API to initiate the download of video content from remote servers,
as well as managing the local content available in the VDS.

The `VDS` functionality can run in low power devices such as raspberry pi and similar SBCs, and is
tailored such that it uses minimal resources during runtime.
