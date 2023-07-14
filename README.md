# demo-bevy_robot
Display UR5 robots using the bevy engine


## native application

### build

Run cmd
```shell
cargo build --release
```

## single page web application

### build

1. Compile to wasm, refer to [trunk](https://trunkrs.dev/). the generated files are in the path "./dist". 
    ```shell
    trunk build --release
    ```

2. Modify the "dist/index.html" file, add code disable the right mouse button menu.
    ```html
    <script type="text/javascript">
        document.oncontextmenu = function(){
            return false;
        }
    </script>
    ```