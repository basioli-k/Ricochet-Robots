<?php 

class LogoutController {

    function index(){
        session_unset();
        session_destroy();

        header( 'Location: index.php' );
        exit();
    }
}

?> 