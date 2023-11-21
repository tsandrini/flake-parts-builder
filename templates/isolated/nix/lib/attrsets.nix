# --- nix/lib/attrsets.nix
{lib, ...}:
with lib;
with builtins; rec {
  /*
  Apply a map to every attribute of an attrset and then filter the resulting
  attrset based on a given predicate function.

  *Type*: `mapFilterAttrs :: (AttrSet b -> Bool) -> (AttrSet a -> AttrSet b) -> AttrSet a -> AttrSet b`
  */
  mapFilterAttrs =
    # (AttrSet b -> Bool) Predicate used for filtering
    pred:
    # (AttrSet a -> AttrSet b) Function used for transforming the given AttrSets
    f:
    # (AttrSet a) Initial attrset
    attrs:
      filterAttrs pred (mapAttrs' f attrs);

  /*
  Recursively merges a list of attrsets.

  *Type*: `mergeAttrs :: [AttrSet] -> AttrSet`

  Example:
  ```nix title="Example" linenums="1"
  mergeAttrs [
   { keyA = 1; keyB = 3; }
   { keyB = 10; keyC = "hey"; nestedKey = { A = null; }; }
   { nestedKey = { A = 3; B = 4; }; }
  ]
  => { keyA = 1; keyB = 10; keyC = "hey"; nestedKey = { A = 3; B = 4; };}
  ```
  */
  mergeAttrs =
    # ([AttrSet]) The list of attrsets
    attrs:
      foldl' (acc: elem: acc // elem) {} attrs;

  /*
  Recursively flattens a nested attrset into a list of just its values.

  *Type*: `flatten :: AttrSet a -> [a]`

  Example:
  ```nix title="Example" linenums="1"
  flatten {
    keyA = 10;
    keyB = "str20";
    keyC = {
      keyD = false;
      keyE = {
        a = 10;
        b = "20";
        c = false;
      };
    };
  }
   => [ 10 "str20" false 10 "20" false ]
  ```
  */
  flatten =
    # (AttrSet a) Initial nested attrset
    attrs:
      collect (x: !isAttrs x) attrs;
}
