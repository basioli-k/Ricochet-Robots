<?php require_once  __SITE_PATH . "/view/_header.php";?>

<div class="wrapper">
<div class="container">
<h1 id="main_heading">Main Menu</h1>

<?php
if(isset($warning))
    if(strlen($warning) > 0)
    {
        echo "<br>";
        echo '<div id="main_error"><b>' . $warning . "</b></div><br>";
    }
?>

<form action="<?php echo __SITE_URL; ?>/index.php">
    <button type="submit" id="play" name="rt" value="gameplay" class="button"> Play game </button><br><br>
    <button type="submit" name="rt" value="stats" class="button"> Stats and leaderboards </button><br><br>
    <button type="submit" name="rt" value="logout" class="button"> Logout </button><br>
</form>


</div>
</div>

<?php require_once __SITE_PATH . "/view/_footer.php";?>