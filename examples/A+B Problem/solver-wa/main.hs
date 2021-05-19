module Main where

main = interact (unlines.map (show .(+1). sum . map read . words).lines)
