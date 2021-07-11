<?php require_once  __SITE_PATH . "/view/_header.php";?>

<div class="wrapper">
<div class="container">
<h1>Main Menu</h1>
<form action="<?php echo __SITE_URL; ?>/index.php">
<ul class="main_menu">
    <li class="main_menu_entry">
        <button type="submit" id="play" name="rt" value="gameplay" class="button"> Play game </button>
    </li>
    <li class="main_menu_entry">
        <button type="submit" name="rt" value="stats" class="button"> Stats and leaderboards </button>
    </li>
    <li class="main_menu_entry">
        <button type="submit" name="rt" value="logout" class="button"> Logout </button>
    </li>
</ul>


</form>
</div>
</div>

<?php require_once __SITE_PATH . "/view/_footer.php";?>