<?php

class LoginController
{
    function index(){
        if(isset($_SESSION["player"]))
            require_once __SITE_PATH . "/view/main_menu.php";
        else
            require_once __SITE_PATH . "/view/login.php";
        exit();
    }

    function verifyLogin(){
        if( !isset( $_POST["username"] ) || !isset( $_POST["password"]) || empty($_POST["password"]  ) )
        {
            unset( $_POST["username"] );
            unset( $_POST["password"] );

            $warning  = "Please enter the username and password.";
            
            require_once __SITE_PATH . "/view/login.php";
            exit();
        }

        if( !preg_match( "/^[a-zA-Z]{3,15}$/", $_POST["username"] ) )
        {
        
            $warning  = "The username has to have between 3 and 15 letters.";
            require_once __SITE_PATH . "/view/login.php";
            
            exit();
        }

        $player = Player::getPlayer(  $_POST["username"] );

        if( $player === false )
        {
            $warning = "The username doesn't exist.";
            require_once __SITE_PATH . "/view/login.php";
            exit();
        }
        else if( $player->has_registered === "0" )
        {
            $warning = "The username didn't register yet. Check your e-mail.";
            require_once __SITE_PATH . "/view/login.php";
            exit();
        }
        else if( !password_verify( $_POST["password"], $player->password_hash ) )
        {
            $warning = "Wrong password. Please try again.";
            require_once __SITE_PATH . "/view/login.php";
            exit();        
        }

        $_SESSION["player"] = $player;
        // $_COOKIE["username"] = $player->username;
        setcookie("username", $player->username, time() + 86400); //24 hours

        require_once __SITE_PATH . "/controller/mainMenuController.php";
        
        $con = new MainMenuController();
        $con->index();
        exit();
    }
}
?>