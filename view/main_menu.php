<?php require_once  __SITE_PATH . "/view/_header.php";?>

<form action="<?php echo __SITE_URL; ?>/index.php">
<ul class="main_menu">
    <li class="main_menu_entry">
        <button type="submit" name="rt" value="gameplay" class="main_menu_button"> Play game </button>
    </li>
    <li class="main_menu_entry">
        <button type="submit" name="rt" value="stats" class="main_menu_button"> Stats and leaderboards </button>
    </li>
    <li class="main_menu_entry">
        <button type="submit" name="rt" value="logout" class="main_menu_button"> Logout </button>
    </li>
</ul>


</form>


<?php require_once __SITE_PATH . "/view/_footer.php";?>