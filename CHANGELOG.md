# Kaolinite Changelog
All dates are in DD/MM/YYYY format. 

This project uses semantic versioning.

<!--
## [X.Y.Z] - DD/MM/YYYY
\+
\~
\-
-->

## [0.3.1] - 06/07/2021
\~ Fixed panic issues in next_word_forth

## [0.3.0] - 06/07/2021
\+ Added cactus: a editor to demonstrate kaolinite

\+ Added support for accessing the line below the document

\+ Added a method to generate line number text

\+ Added support for tab rendering

\+ Added methods for finding the next and previous word index

\+ Added functions to help with display widths

\+ Added file type lookup function to determine type from file extension

\~ Fixed issues with removing

\~ Fixed issues with splicing up

\~ Used char indices instead of display indices

\~ Fixed the EOI issues

\~ Followed clippy lints

## [0.2.1] - 30/06/2021
\~ Row linking optimisation (~1.33x faster)

## [0.2.0] - 30/06/2021
\~ Text removal optimisation (~1.25x faster)

\- Only allowed inclusive and exclusive ranges in Row::remove to prevent spaghettification

## [0.1.5] - 30/06/2021
\~ Row splicing optimisation (~1.2x faster)

## [0.1.4] - 30/06/2021
\~ More huge optimisation (~5x faster)

## [0.1.3] - 30/06/2021
\~ Huge optimisation (~3x faster)

## [0.1.2] - 29/06/2021
\+ Added benchmark

## [0.1.1] - 29/06/2021
\+ Added changelog

## [0.1.0] - 29/06/2021
\+ Initial release
